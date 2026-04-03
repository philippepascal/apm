+++
id = "4cec7a17"
title = "apm-core: collaborators config, local.toml, and identity resolution"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "philippepascal"
branch = "ticket/4cec7a17-apm-core-collaborators-config-local-toml"
created_at = "2026-04-02T20:53:47.546444Z"
updated_at = "2026-04-03T23:45:21.654589Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

There is no concept of stable human identity in apm-core. The `author` field is currently set to the `APM_AGENT_NAME` environment variable (an ephemeral agent session string like `claude-0402-1430-a3f9`) or the literal string `"apm"` for automated actions. There is no collaborators list in config, no per-machine identity file, and no resolution function. As a result, there is no reliable way to track which human created a ticket — only which short-lived agent process ran at the time.

The desired state (design doc points 1–3) is:
- `.apm/config.toml` carries a `collaborators` list under `[project]`
- `.apm/local.toml` (gitignored, per-machine) stores the local user's `username`
- A `resolve_identity(repo_root)` function returns the username from `local.toml` or `"unassigned"` as a fallback
- `apm new` uses the resolved identity as `author` instead of the agent env var
- `apm init` prompts for a username, writes `local.toml`, updates `collaborators`, and adds `local.toml` to `.gitignore`
- The `agent` field is dropped from frontmatter writes (silently tolerated on read for backward compatibility)

### Acceptance criteria

- [ ] `ProjectConfig` has a `collaborators: Vec<String>` field (default: empty) that parses from `[project] collaborators = ["alice", "bob"]` in `.apm/config.toml`
- [ ] A `LocalConfig` struct with an optional `username` field loads from `.apm/local.toml`; if the file is absent, loading returns a default (no error)
- [ ] `resolve_identity(repo_root)` returns the `username` from `.apm/local.toml` when present and non-empty, and returns `"unassigned"` otherwise
- [ ] `apm new` sets `author` to the value returned by `resolve_identity` instead of `APM_AGENT_NAME`
- [ ] `apm init` (interactive TTY) prompts "What is your username?", writes the answer to `.apm/local.toml` as `username = "..."`
- [ ] `apm init` adds the entered username to `collaborators` in the newly created `.apm/config.toml`
- [ ] `apm init` adds `.apm/local.toml` to `.gitignore`
- [ ] `apm init` (non-interactive / no TTY) skips the username prompt and does not write `.apm/local.toml`
- [ ] Existing ticket files with `agent = "..."` in frontmatter parse without error; new tickets written by `apm` do not include an `agent` field

### Out of scope

- Git host plugin identity resolution (DESIGN-users.md point 4) — GitHub API / `gh auth status` as identity source
- WebAuthn / OTP auth, `apm register`, `apm sessions`, `apm revoke` (point 5)
- `apm list --mine`, `--author`, and `--unassigned` filter changes (point 7)
- `/api/me` endpoint and UI author filter (point 8)
- Distribution / packaging (point 6)
- `assignee` field — deferred per design doc
- Rewriting existing ticket files to replace legacy `author` values — existing values are left as-is
- Validating `author` against the collaborators list at `apm new` time — warn-only is deferred to a later ticket

### Approach

**1. `apm-core/src/config.rs` — collaborators and LocalConfig**

Add `collaborators: Vec<String>` to `ProjectConfig` with `#[serde(default)]`.

Add `LocalConfig` struct with a `username: Option<String>` field and a `load(repo_root)` method that reads `.apm/local.toml`, returning a default if absent.

Add `resolve_identity(repo_root: &Path) -> String` that returns the non-empty username from `LocalConfig`, or `"unassigned"` as fallback.

**2. `apm-core/src/ticket.rs` — drop agent from writes**

Change the `agent` field in `Frontmatter` from `#[serde(skip_serializing_if = "Option::is_none")]` to `#[serde(default, skip_serializing)]`. Existing files with `agent = "..."` still parse without error (value is read but never re-written).

**3. `apm/src/cmd/new.rs` — use resolved identity**

Replace the `APM_AGENT_NAME` env-var lookup with a call to `apm_core::config::resolve_identity(root)`.

**4. `apm-core/src/init.rs` — prompt, local.toml, gitignore**

a) Gitignore: add `.apm/local.toml` to the `entries` array in `ensure_gitignore`.

b) Add `prompt_username() -> Result<String>` helper (mirrors `prompt_project_info`). Returns the trimmed input.

c) Add `write_local_config(root: &Path, username: &str) -> Result<()>` that writes `username = "<username>"` to `.apm/local.toml`.

d) Add `username: Option<&str>` parameter to `default_config`. When `Some`, include `collaborators = ["<username>"]` in the `[project]` section; when `None`, omit it.

e) In `setup()`: TTY path — after the existing project-info prompt, call `prompt_username()`, pass the result to `default_config` and `write_local_config`. Non-TTY path — pass `None` to `default_config`, skip writing `local.toml`. Collaborators are only written into freshly-created config; `local.toml` is always safe to write/overwrite (gitignored).

**Order of changes**

1. `config.rs`: `collaborators`, `LocalConfig`, `resolve_identity`
2. `ticket.rs`: `agent` serde attributes
3. `init.rs`: gitignore entry, prompt, write helpers, `default_config` signature
4. `cmd/new.rs`: swap author source
5. Tests: unit tests for `LocalConfig::load` (file present / absent / empty username), `resolve_identity`, collaborators parse, gitignore includes `.apm/local.toml`; integration test confirming `apm new` sets `author` from `local.toml`

### 1. `apm-core/src/config.rs` — add collaborators and LocalConfig

Add `collaborators: Vec<String>` to `ProjectConfig` with `#[serde(default)]`.

Add `LocalConfig` struct with a `username: Option<String>` field and a `load(repo_root)` method that reads `.apm/local.toml`, returning a default if absent.

Add `resolve_identity(repo_root: &Path) -> String` that returns the non-empty username from `LocalConfig`, or `"unassigned"` as fallback.

### 2. `apm-core/src/ticket.rs` — drop agent from writes

Change the `agent` field in `Frontmatter` from `#[serde(skip_serializing_if = "Option::is_none")]` to `#[serde(default, skip_serializing)]`. This means existing files with `agent = "..."` parse without error (value still deserialised into the field) but `agent` is never written on next save.

### 3. `apm/src/cmd/new.rs` — use resolved identity

Replace the `APM_AGENT_NAME` env-var lookup with a call to `apm_core::config::resolve_identity(root)`.

### 4. `apm-core/src/init.rs` — prompt, local.toml, gitignore

a) Gitignore: add `.apm/local.toml` to the `entries` array in `ensure_gitignore`.

b) Add `prompt_username() -> Result<String>` helper (mirrors `prompt_project_info`).

c) Add `write_local_config(root: &Path, username: &str) -> Result<()>` that writes `username = "<username>"` to `.apm/local.toml`.

d) Add `username: Option<&str>` parameter to `default_config`. When `Some`, include `collaborators = ["<username>"]` in the `[project]` section; when `None`, omit it.

e) In `setup()`: after the project-info prompt block (TTY path), call `prompt_username()` and pass the result to both `default_config` and `write_local_config`. In the non-TTY path, pass `None` to `default_config` and skip writing `local.toml`. Writing `local.toml` happens regardless of whether config already exists (it is gitignored and safe to overwrite); adding `collaborators` to the config only happens on first creation when config is freshly written.

### Order of changes

1. `config.rs`: `collaborators`, `LocalConfig`, `resolve_identity`
2. `ticket.rs`: `agent` serde attributes
3. `init.rs`: gitignore entry, prompt, write helpers, `default_config` signature
4. `cmd/new.rs`: swap author source
5. Tests: unit tests for `LocalConfig::load` (file present / absent / empty username), `resolve_identity`, collaborators parse, gitignore includes `.apm/local.toml`; integration test confirming `apm new` sets `author` from `local.toml`

### Open questions


### Amendment requests

- [ ] `LocalConfig` already exists in `config.rs` (added by e1582fd0 for worker spawn config) with a `workers: LocalWorkersOverride` field. This ticket must **extend** the existing `LocalConfig` by adding `username: Option<String>` to it, not create a new struct. Update the approach accordingly.
- [ ] `.apm/local.toml` is already in `.gitignore` (added by e1582fd0). Remove the gitignore AC item and update the approach to note this is already done.
- [ ] `Config::load` already reads and merges `.apm/local.toml` for worker overrides (added by e1582fd0). The `username` field will be available automatically through the existing `LocalConfig` load path — no new file-reading code needed.
- [ ] The `apm init` AC items (prompt for username, write local.toml, add to gitignore) overlap with ticket 79326024 which is specifically scoped to init changes. Clarify: this ticket adds the data structures and `resolve_identity()` only; all init-flow changes are in 79326024.
- [ ] Add `.apm/sessions.json` and `.apm/credentials.json` to the `ensure_gitignore` entries array — these files are created by e2e3d958 and 8a08637c respectively and must not be committed.
- [ ] Set effort and risk to non-zero values.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:21Z | new | groomed | apm |
| 2026-04-02T23:21Z | groomed | in_design | philippepascal |
| 2026-04-02T23:25Z | in_design | specd | claude-0402-2321-b7f2 |
| 2026-04-03T23:42Z | specd | ammend | apm |
| 2026-04-03T23:45Z | ammend | in_design | philippepascal |
