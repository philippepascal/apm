+++
id = "0038"
title = "Docker sandbox for apm worker processes"
state = "closed"
priority = 3
effort = 5
risk = 2
author = "claude-0327-1854-10aa"
agent = "88722"
branch = "ticket/0038-docker-sandbox-for-apm-worker-processes"
created_at = "2026-03-28T07:32:08.465132Z"
updated_at = "2026-03-30T19:54:40.308264Z"
+++

## Spec

### Problem

When `apm start` spawns a `claude` subprocess (ticket #37), the worker process
inherits the full filesystem access of the user running it. A buggy or confused
worker could in principle modify files outside its worktree.

The first instinct is to mount credential dotfiles (`~/.claude`, `~/.ssh`,
`~/.config/gh`) into a Docker container for isolation. But this trades one
problem for another: those dotfiles contain long-lived secrets stored in
plaintext. Mounting them into a container makes them readable by anything
running inside — including the worker itself, any dependency it installs, and
anything that exploits a container escape. The dotfile model is the default
because it is convenient, not because it is safe.

The right model is:

1. Run the worker inside a Docker container so the host filesystem is not
   accessible (only the worktree is mounted).
2. Inject only the credentials the worker actually needs as environment
   variables — never as mounted files — so secrets are never written to the
   container's filesystem.
3. Derive those values at spawn time from the OS keychain or environment,
   keeping long-lived keys out of `apm.toml` and off disk.

**The worker's credential footprint is minimal.** Per ticket #37, the worker
only does local `git commit` — it never pushes, never creates PRs. Those
operations happen on the host after the container exits, using credentials
that never enter the container. The worker therefore needs only:
- `ANTHROPIC_API_KEY` — for the claude CLI to call the Anthropic API
- `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL` — so commits have the right identity

That's it. `GH_TOKEN` stays on the host.

This is opt-in. Users who do not configure `[workers] container` get the
current native behaviour unchanged.

### Acceptance criteria

- [x] `apm.toml` supports a `[workers]` section with `container = "<image>"`;
  when absent or empty, native spawning (ticket #37) is used unchanged
- [x] When `container` is set, `apm start` runs the worker via
  `docker run --rm` with only the worktree mounted at `/workspace` (read-write)
  and nothing else from the host filesystem
- [x] Credentials are injected as environment variables, never as volume mounts.
  The worker needs only:
  - `ANTHROPIC_API_KEY` — required for the claude CLI
  - `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`,
    `GIT_COMMITTER_EMAIL` — so commits have the right identity
  (`GH_TOKEN` is NOT injected — the worker never pushes or creates PRs;
  those are handled by `apm start` on the host after the container exits)
- [x] `apm start` resolves each credential at spawn time using this priority:
  1. Environment variable already set in the caller's shell
  2. macOS Keychain (`security find-generic-password -s "<service>" -w`)
     using configurable service names in `[workers.keychain]`
  3. Hard failure with a clear error if a required credential cannot be found
- [x] `apm.toml` `[workers.keychain]` lets users map credential names to
  Keychain service names, e.g.:
  ```toml
  [workers.keychain]
  ANTHROPIC_API_KEY = "anthropic-api-key"
  ```
- [x] `apm init --with-docker` generates a `Dockerfile.apm-worker` into `.apm/`
  and prints instructions to build it; it does NOT auto-run `docker build`
- [x] The generated `Dockerfile.apm-worker` installs: `claude` CLI, `git`,
  and `apm` (from the project's own binary or a downloaded release); it does
  NOT include `gh` — the worker never pushes or creates PRs; it includes
  commented sections for users to add project-specific dependencies
- [x] The worker container is ephemeral: started with `--rm`, no persistent
  volumes or named containers
- [x] `apm validate` checks: if `[workers] container` is set but `docker` is not
  in PATH, print a warning

### Out of scope

- Building or pushing Docker images (user does that manually from the
  generated `Dockerfile.apm-worker`)
- Linux keychain / `libsecret` integration (macOS Keychain only for now;
  Linux users pass credentials via environment variables)
- Container networking restrictions — the worker needs outbound access to the
  Anthropic API; `--network none` is therefore not used
- Multi-platform image builds or image version pinning in `apm.toml`
- Windows support for the container path
- Secret rotation or TTL enforcement (that is the user's responsibility)
- Secrets manager integrations (Vault, 1Password CLI, AWS Secrets Manager) —
  the keychain abstraction is the extension point for those

### Approach

**Config** (`apm-core/src/config.rs`):

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WorkersConfig {
    pub container: Option<String>,
    #[serde(default)]
    pub keychain: std::collections::HashMap<String, String>,
}
```

Add `workers: WorkersConfig` to `Config`.

**Credential resolution** (`apm-core/src/credentials.rs`, new file):

```rust
pub fn resolve(name: &str, keychain_service: Option<&str>) -> anyhow::Result<String> {
    // 1. Check environment
    if let Ok(v) = std::env::var(name) {
        if !v.is_empty() { return Ok(v); }
    }
    // 2. macOS Keychain
    #[cfg(target_os = "macos")]
    if let Some(service) = keychain_service {
        let out = std::process::Command::new("security")
            .args(["find-generic-password", "-s", service, "-w"])
            .output()?;
        if out.status.success() {
            let val = String::from_utf8(out.stdout)?.trim().to_string();
            if !val.is_empty() { return Ok(val); }
        }
    }
    anyhow::bail!("credential {name:?} not found in environment or keychain");
}
```

**`apm start` spawn path** (`apm/src/cmd/start.rs`):

```
if let Some(image) = &config.workers.container {
    spawn_container_worker(root, wt, ticket, image, &config.workers.keychain, agent_name)
} else {
    spawn_native_worker(root, wt, ticket, agent_name)  // ticket #37
}
```

`spawn_container_worker` builds:
```
docker run --rm
  --volume <wt>:/workspace
  --workdir /workspace
  --env ANTHROPIC_API_KEY=<resolved>
  --env GIT_AUTHOR_NAME=<from git config or env>
  --env GIT_AUTHOR_EMAIL=<from git config or env>
  --env GIT_COMMITTER_NAME=<same>
  --env GIT_COMMITTER_EMAIL=<same>
  --env APM_AGENT_NAME=<generated>
  <image>
  claude --dangerously-skip-permissions -p "<ticket>" --system "<worker md>"
```

The container has no network git credentials and no `GH_TOKEN`. The worker
only does local `git commit`. After the container exits, `apm state <id> implemented`
(running on the host, with full credentials) handles `git push` and `gh pr create`
via the `completion` property on the transition.

Git identity falls back to `git config user.name` / `git config user.email`
from the host if the env vars are not set.

**`Dockerfile.apm-worker` template** (generated by `apm init --with-docker`):

```dockerfile
FROM rust:1.82-slim

# System tools
RUN apt-get update && apt-get install -y \
    curl git unzip ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Claude CLI
RUN curl -fsSL https://storage.googleapis.com/anthropic-claude-cli/install.sh | sh

# apm binary (replace with your version)
COPY target/release/apm /usr/local/bin/apm

# Add project-specific dependencies here:
# RUN apt-get install -y nodejs npm   # for Node projects
# RUN pip install -r requirements.txt # for Python projects

# gh CLI is NOT needed — the worker only runs local git commits;
# push and PR creation happen on the host via apm state <id> implemented.

WORKDIR /workspace
```

### Amendment requests

- [x] The worker never pushes the branch or creates a PR. Under the new design
  (`completion` property on transitions), `apm state <id> implemented` handles
  push and PR creation on the host. Remove all references to `apm start`
  doing post-container push/PR. Update the credential list: `GH_TOKEN` is
  confirmed not needed, but make this explicit in the AC and Dockerfile.
- [x] Remove `gh` CLI from the `Dockerfile.apm-worker` template — it is not
  needed inside the container.
- [x] `apm init --with-docker` references need to align with the new `apm init`
  design (`.apm/` folder). The Dockerfile should be generated into `.apm/` or
  documented alongside other init outputs.

**Audited 2026-03-29:** Approach still valid. `apm-core/src/config.rs` has no `WorkersConfig` yet. `apm/src/cmd/start.rs` has native spawn support but no Docker path. `apm-core/src/credentials.rs` does not exist yet. All file paths and function names referenced in the approach are accurate. Previous audit corrected `apm verify` → `apm validate` throughout.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T07:32Z | — | new | claude-0327-1854-10aa |
| 2026-03-28T07:34Z | new | specd | claude-0327-1854-10aa |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |
| 2026-03-29T20:39Z | ammend | in_design | claude-0329-main |
| 2026-03-29T20:42Z | in_design | specd | claude-0329-main |
| 2026-03-30T19:21Z | specd | ready | apm |
| 2026-03-30T19:23Z | ready | in_progress | philippepascal |
| 2026-03-30T19:34Z | in_progress | implemented | claude-0330-1930-b7e2 |
| 2026-03-30T19:47Z | implemented | accepted | apm-sync |
| 2026-03-30T19:54Z | accepted | closed | apm-sync |