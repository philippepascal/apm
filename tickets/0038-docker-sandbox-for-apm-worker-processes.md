+++
id = 38
title = "Docker sandbox for apm worker processes"
state = "new"
priority = 0
effort = 5
risk = 0
author = "claude-0327-1854-10aa"
branch = "ticket/0038-docker-sandbox-for-apm-worker-processes"
created_at = "2026-03-28T07:32:08.465132Z"
updated_at = "2026-03-28T07:34:58.310362Z"
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
2. Inject credentials as short-lived, scoped environment variables — not as
   mounted files — so secrets are never written to the container's filesystem.
3. Derive those values at spawn time from the OS keychain or environment,
   keeping long-lived keys out of `apm.toml` and off disk.

This is opt-in. Users who do not configure `[workers] container` get the
current native behaviour unchanged.

### Acceptance criteria

- [ ] `apm.toml` supports a `[workers]` section with `container = "<image>"`;
  when absent or empty, native spawning (ticket #37) is used unchanged
- [ ] When `container` is set, `apm start` runs the worker via
  `docker run --rm` with only the worktree mounted at `/workspace` (read-write)
  and nothing else from the host filesystem
- [ ] Credentials are injected as environment variables, never as volume mounts:
  - `ANTHROPIC_API_KEY` — required for the claude CLI
  - `GH_TOKEN` — used by `gh` for PR creation and by git for HTTPS auth
  - `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`,
    `GIT_COMMITTER_EMAIL` — so commits have the right identity
- [ ] `apm start` resolves each credential at spawn time using this priority:
  1. Environment variable already set in the caller's shell
  2. macOS Keychain (`security find-generic-password -s "<service>" -w`)
     using configurable service names in `[workers.keychain]`
  3. Hard failure with a clear error if a required credential cannot be found
- [ ] `apm.toml` `[workers.keychain]` lets users map credential names to
  Keychain service names, e.g.:
  ```toml
  [workers.keychain]
  ANTHROPIC_API_KEY = "anthropic-api-key"
  GH_TOKEN          = "github-token"
  ```
- [ ] `apm init --with-docker` generates a `Dockerfile.apm-worker` at the repo
  root and prints instructions to build it; it does NOT auto-run `docker build`
- [ ] The generated `Dockerfile.apm-worker` installs: `claude` CLI, `git`,
  `gh`, `apm` (from the project's own binary or a downloaded release), and
  `cargo`/`rustup`; it includes commented sections for users to add
  project-specific dependencies
- [ ] Git HTTPS auth uses `GH_TOKEN` via git credential helper inside the
  container — no SSH key mounting required or supported in the default image
- [ ] The worker container is ephemeral: started with `--rm`, no persistent
  volumes or named containers
- [ ] `apm verify` checks: if `[workers] container` is set but `docker` is not
  in PATH, print a warning

### Out of scope

- Building or pushing Docker images (user does that manually from the
  generated `Dockerfile.apm-worker`)
- Linux keychain / `libsecret` integration (macOS Keychain only for now;
  Linux users pass credentials via environment variables)
- Container networking restrictions (no `--network none`; the worker needs
  outbound access to GitHub and the Anthropic API)
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
  --env GH_TOKEN=<resolved>
  --env GIT_AUTHOR_NAME=<from git config or env>
  --env GIT_AUTHOR_EMAIL=<from git config or env>
  --env GIT_COMMITTER_NAME=<same>
  --env GIT_COMMITTER_EMAIL=<same>
  --env APM_AGENT_NAME=<generated>
  <image>
  claude --dangerously-skip-permissions -p "<ticket>" --system "<worker md>"
```

Git identity falls back to `git config user.name` / `git config user.email`
from the host if the env vars are not set.

**`Dockerfile.apm-worker` template** (generated by `apm init --with-docker`):

```dockerfile
FROM rust:1.82-slim

# System tools
RUN apt-get update && apt-get install -y \
    curl git unzip ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [signed-by=...] https://cli.github.com/packages stable main" \
    > /etc/apt/sources.list.d/github-cli.list && \
    apt-get update && apt-get install -y gh

# Claude CLI
RUN curl -fsSL https://storage.googleapis.com/anthropic-claude-cli/install.sh | sh

# apm binary (replace with your version)
COPY target/release/apm /usr/local/bin/apm

# Configure git to use GH_TOKEN for HTTPS auth
RUN git config --global credential.helper \
    '!f() { echo username=x-token; echo password=$GH_TOKEN; }; f'

# Add project-specific dependencies here:
# RUN apt-get install -y nodejs npm   # for Node projects
# RUN pip install -r requirements.txt # for Python projects

WORKDIR /workspace
```

## History

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T07:32Z | — | new | claude-0327-1854-10aa |