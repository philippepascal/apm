+++
id = "79326024"
title = "apm init: username prompt, local.toml, gitignore, and collaborators bootstrap"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "44684"
branch = "ticket/79326024-apm-init-username-prompt-local-toml-giti"
created_at = "2026-04-02T20:53:51.576153Z"
updated_at = "2026-04-02T23:26:11.560729Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

The `apm init` command does not prompt for a username, does not write `.apm/local.toml`, does not add `.apm/local.toml` to `.gitignore`, and does not seed a `collaborators` list in `.apm/config.toml`. As a result, a freshly initialised project has no local identity configured and no collaborator roster to build from.

Per DESIGN-users.md points 1 and 7, each machine should carry a gitignored `.apm/local.toml` with the developer's `username`, the tracked `.apm/config.toml` should include a `collaborators` list seeded with that username, and `.apm/local.toml` must be excluded from version control so it is never accidentally committed.

Ticket #4cec7a17 (dependency) adds the `LocalConfig` struct, the `collaborators` field on `ProjectConfig`, and `resolve_identity()` to `apm-core`. This ticket wires those building blocks into the `apm init` flow: the interactive prompt, the file writes, and the gitignore update.

### Acceptance criteria

- [ ] After `apm init` in an interactive TTY, `.apm/local.toml` is created containing `username = "<entered-value>"`
- [ ] After `apm init` in a non-interactive environment (no TTY), `.apm/local.toml` is not created
- [ ] After `apm init`, `.gitignore` contains the entry `.apm/local.toml`
- [ ] After `apm init` in a TTY on a fresh project (config.toml not yet written), `.apm/config.toml` `[project]` section contains `collaborators = ["<username>"]`
- [ ] After `apm init` in a non-TTY on a fresh project, `collaborators` in the written config defaults to an empty array `[]` (field is present but empty)
- [ ] Running `apm init` a second time when `.apm/local.toml` already exists does not overwrite or re-prompt for username
- [ ] Running `apm init` a second time does not duplicate the `.apm/local.toml` entry in `.gitignore`
- [ ] `ensure_gitignore` is idempotent: calling it twice produces the same `.gitignore` content

### Out of scope

- `LocalConfig`, `resolve_identity()`, and `ProjectConfig.collaborators` data structures — those are implemented in ticket #4cec7a17
- Dropping `agent` from frontmatter writes — also ticket #4cec7a17
- Using `resolve_identity()` in `apm new` — also ticket #4cec7a17
- Git host plugin (GitHub API) identity resolution — DESIGN-users.md point 4, future ticket
- Validating username against the collaborators list at `apm new` time — deferred, warn-only per design doc
- WebAuthn / OTP / `apm register` / `apm sessions` / `apm revoke` — DESIGN-users.md point 5, separate tickets
- `apm list --mine` / `--author` / `--unassigned` filter additions — DESIGN-users.md point 7, separate ticket
- `/api/me` endpoint and UI author filter — DESIGN-users.md point 8, separate ticket
- Manually adding collaborators to an existing project (post-init) — out of scope for init bootstrapping

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:19Z | new | groomed | apm |
| 2026-04-02T23:26Z | groomed | in_design | philippepascal |