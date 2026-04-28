+++
id = "d486d183"
title = "apm help config: render config.toml schema from Config struct"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d486d183-apm-help-config-render-config-toml-schem"
created_at = "2026-04-28T19:27:57.393396Z"
updated_at = "2026-04-28T19:27:57.393396Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

Replace the `render_config()` stub from ticket bc89e0a0 with a real renderer that uses the auto-derive infrastructure from ticket 069c3403 to render the `Config` struct from `apm-core/src/config.rs`.

**Sections to cover** (top-level keys in `.apm/config.toml`):
- `[project]` — name, description, default_branch
- `[tickets]` — dir, archive_dir
- `[worktrees]` — dir, agent_dirs
- `[git_host]` — provider, repo, token_env
- `[agents]` — instructions, max_concurrent, max_workers_per_epic, max_workers_on_default, side_tickets, skip_permissions
- `[sync]` — aggressive
- `[logging]` — enabled, file
- `[workers]` — command, args, model, container, image, etc.
- `[worker_profiles.<name>]` — same fields as workers, plus instructions, role_prefix

**Output structure:**
- Group fields by their TOML section header.
- Per field: name, type, default (when any), one-line description from doc comments.
- Render order matches struct definition order or section grouping in `config.toml`.

**Implementation pointers:**
- In `apm/src/cmd/help.rs`: replace the stub for `config` topic. Call into `apm_core::help_schema` for the `Config` type.
- Doc comments on `Config` and its nested structs in `apm-core/src/config.rs` may need to be added or improved as part of this ticket — describe each field in one line.

**Out of scope:**
- Worker_profiles content beyond a generic description (each profile inherits the workers fields; documenting each user-defined profile is meaningless).
- Examples beyond what the struct doc comments contain.
- Format conversion (TOML <-> JSON).

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
