+++
id = "76dc81c5"
title = "surface implicit configurations in default config"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/76dc81c5-surface-implicit-configurations-in-defau"
created_at = "2026-05-24T19:24:15.489361Z"
updated_at = "2026-05-24T21:26:07.219937Z"
+++

## Spec

### Problem

When `apm init` runs, it writes `.apm/config.toml` with only the parameters needed to get started: project identity, ticket and worktree paths, and a few agent/worker knobs. Eight additional configuration sections — `[sync]`, `[git_host]`, `[server]`, `[context]`, `[isolation]`, `[work]` — and several optional fields within the sections that _are_ shown (`[agents].side_tickets`, `[agents].skip_permissions`, `[workers].container`, `[workers].env`, `[workers].keychain`) are written without any mention, with their defaults in effect but invisible.

A new user inspecting the freshly-written config has no way to discover these knobs without reading the Rust source or searching documentation. The fix is to include every implicit parameter in the generated file as commented-out TOML, each annotated with its default value and a one-line description — the pattern used by many well-known tools (Cargo, Redis, Postgres). The file stays functional as-is; the comments are a self-contained reference.

### Acceptance criteria

- [x] `apm init` on a fresh repo produces a `config.toml` that includes commented-out stubs for `[sync]`, `[git_host]`, `[server]`, `[context]`, `[isolation]`, and `[work]`
- [x] The `[agents]` block in the generated config includes commented-out `side_tickets` and `skip_permissions` lines with their default values
- [x] The `[workers]` block in the generated config includes commented-out `container`, `env`, and `keychain` lines
- [x] Every commented-out parameter shows its default value (or an illustrative example for parameters with no scalar default, such as `container`)
- [x] Every commented-out parameter is accompanied by a short inline comment (`# …`) describing its effect
- [x] The generated `config.toml` is valid TOML when all comment lines are removed
- [x] Running `apm init` a second time on an already-initialised repo does not produce a `.init` diff file (idempotency preserved)
- [x] Existing tests in `apm-core` pass without modification after the change

### Out of scope

- Changes to `workflow.toml` or `ticket.toml` — they have their own files and are already dense
- Changes to `local.toml` — machine-specific, intentionally minimal
- An interactive wizard or `apm config` command for setting values
- Documentation outside the generated `config.toml` itself
- Surfacing `LocalConfig` fields (`[workers].command`, `[workers].args`) in `config.toml` — those belong in `local.toml`

### Approach

The entire change lives in `default_config()` in `apm-core/src/init.rs`. Expand the format string to include commented-out stubs for every implicit parameter, organised by section.

#### Additions within existing sections

In the `[agents]` block, add after the existing explicit fields:
```toml
# side_tickets = true        # allow workers to file side-note tickets
# skip_permissions = false   # skip Claude Code permission prompts in workers
```

In the `[workers]` block, add after `model`:
```toml
# container = "apm-worker"   # Docker image for worker agents; omit for local execution
# env = {}                   # environment variables injected into every worker
# keychain = {}              # macOS Keychain items resolved at worker launch (secret_name = keychain_item)
```

#### New commented-out sections (appended after `[logging]`)

```toml
# [sync]
# aggressive = true   # fetch all remote branches before checking state

# [git_host]
# provider = "github"           # git host provider; only "github" is supported
# repo = "owner/repo"           # repository path for PR creation and collaborator lookup
# token_env = "GITHUB_TOKEN"    # env var holding the API token

# [server]
# origin = "http://localhost:3000"    # public-facing URL used in PR descriptions
# url    = "http://127.0.0.1:3000"   # internal URL the CLI uses to reach apm-server

# [context]
# epic_sibling_cap = 20      # max sibling tickets included in worker context bundles
# epic_byte_cap    = 8192    # max byte size of the context bundle injected into worker prompts

# [isolation]
# read_allow = ["/etc/resolv.conf", "~/.gitconfig"]   # paths workers may read outside the worktree
# enforce_worktree_isolation = false                   # block writes outside APM_TICKET_WORKTREE

# [work]
# epic = ""   # default epic ID assigned when creating new tickets with `apm new`
```

#### Idempotency

`default_config()` is a pure function called on every `apm init` run and its output is compared byte-for-byte against the on-disk file by `write_default()`. Because the new comments are part of the generated string, a freshly-initialised config already contains them; a second `apm init` call produces an identical string and no `.init` diff is written. No changes to `write_default()` or the idempotency logic are needed.

#### Tests

All existing tests in `apm-core/src/init.rs` check for substrings or valid-TOML properties that remain true after the change. No test modifications are required. Verify with `cargo test --workspace`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T19:24Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
| 2026-05-24T19:57Z | groomed | in_design | philippepascal |
| 2026-05-24T19:59Z | in_design | specd | claude |
| 2026-05-24T20:30Z | specd | ready | philippepascal |
| 2026-05-24T20:30Z | ready | in_progress | philippepascal |
| 2026-05-24T20:33Z | in_progress | implemented | claude |
| 2026-05-24T21:26Z | implemented | closed | philippepascal(apm-sync) |
