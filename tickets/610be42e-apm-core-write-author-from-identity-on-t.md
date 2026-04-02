+++
id = "610be42e"
title = "apm-core: write author from identity on ticket creation, remove agent field"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "69160"
branch = "ticket/610be42e-apm-core-write-author-from-identity-on-t"
created_at = "2026-04-02T20:53:55.085303Z"
updated_at = "2026-04-02T23:33:12.675117Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

New tickets set `author` from the `APM_AGENT_NAME` environment variable (or fall back to `"apm"`), conflating the ephemeral worker name with a permanent creator identity. Meanwhile, the `agent` frontmatter field tracks the current worker name — but workers are single-use, resumability does not depend on it, and tying frontmatter to a specific naming convention is the wrong direction (DESIGN-users.md point 2).

There is no mechanism today to resolve a real human username for `author`. The design calls for reading `.apm/local.toml` (a gitignored, per-machine file) for the `username` key, falling back to `"apm"` when absent (DESIGN-users.md points 1 and 3).

This ticket adds the identity-resolution function in `apm-core`, wires it into `apm new`, and removes the `agent` field from frontmatter writes and from `apm list`/`apm show` output.

### Acceptance criteria

- [ ] `apm new` sets `author` to the `username` value from `.apm/local.toml` when that file exists and contains a non-empty `username` key
- [ ] `apm new` sets `author` to `"apm"` when `.apm/local.toml` is absent or contains no `username` key
- [ ] Ticket files written by `apm new` do not contain an `agent` key in their TOML frontmatter
- [ ] Existing ticket files that contain an `agent` field are parsed without error (backward-compatible read)
- [ ] `apm list` output no longer includes an `agent=` column
- [ ] `apm show` output no longer prints an `agent:` line
- [ ] `apm list --unassigned` returns tickets where `author == "unassigned"` (was: `agent == null`)
- [ ] `apm take` completes without error on tickets that have no `agent` field

### Out of scope

- Git host plugin identity resolution (DESIGN-users.md point 4) — no GitHub API calls
- `apm init` prompting for username and writing `.apm/local.toml` (a later ticket)
- Validating `author` against a collaborators list at `apm new` time
- `apm list --mine` and `apm list --author` filter flags
- `apm epic new` writing `author` (mirrors ticket behaviour, separate change)
- UI/server changes (author display, board filters, `/api/me` endpoint)

### Approach

**New file: `apm-core/src/identity.rs`**
Add `pub fn resolve_current_user(root: &Path) -> String`:
- Read `.apm/local.toml`; parse with `toml::from_str` into a minimal struct `{ username: Option<String> }`
- Return `username` if present and non-empty
- Return `"apm"` as fallback (covers CI, missing file, empty value)
Add `pub mod identity;` to `apm-core/src/lib.rs`.

**`apm-core/src/ticket.rs` — Frontmatter.agent**
- Change the `agent` field to `#[serde(default, skip_serializing)]` so existing files with `agent = "..."` still parse, but new serializations omit the key entirely.
- `handoff()`: agent is no longer required in frontmatter. Change the guard from `bail!("no agent assigned")` to proceed unconditionally — use `"unknown"` as the old-agent placeholder in the history row when `agent` is `None`.
- `list_filtered()`: change the `--unassigned` predicate from `fm.agent.is_none()` to `fm.author.as_deref() == Some("unassigned")`.

**`apm-core/src/start.rs`**
- Remove `t.frontmatter.agent = Some(agent_name.to_string());`.

**`apm/src/cmd/new.rs`**
- Replace the `APM_AGENT_NAME` env-var lookup with `apm_core::identity::resolve_current_user(root)`.

**`apm/src/cmd/list.rs`**
- Remove the `agent` variable and `agent=…` segment from the `println!` format string.

**`apm/src/cmd/show.rs`**
- Remove the `if let Some(a) = &fm.agent { println!("agent:    {a}"); }` line.

**`apm-core/src/init.rs`**
- Add `".apm/local.toml"` to the `entries` array in `ensure_gitignore()` so `apm init` gitignores it even before the identity-setup ticket ships.

**Tests**
- Unit tests for `identity::resolve_current_user` in `apm-core/src/identity.rs`: covers file absent, file present with username, file present without username key.
- Update existing tests in `ticket.rs` that construct `Frontmatter` with `agent` — remove or adjust the `agent` field.
- Update `handoff` tests to cover the None-agent path.
- All tests pass under `cargo test --workspace`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:30Z | groomed | in_design | philippepascal |