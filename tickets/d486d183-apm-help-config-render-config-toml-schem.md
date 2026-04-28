+++
id = "d486d183"
title = "apm help config: render config.toml schema from Config struct"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d486d183-apm-help-config-render-config-toml-schem"
created_at = "2026-04-28T19:27:57.393396Z"
updated_at = "2026-04-28T19:49:08.971160Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_config()` function in `apm/src/cmd/help.rs` is introduced as a stub by ticket bc89e0a0. It returns a placeholder string referencing this ticket (d486d183). As a result, `apm help config` gives users no actionable information about what fields are valid in `.apm/config.toml`, their types, defaults, or purpose.

The `Config` struct in `apm-core/src/config.rs` already defines all config keys and their types, but nearly every field lacks a `/// doc comment`. Ticket 069c3403 provides `apm_core::help_schema::schema_entries::<T>()` and `render_schema::<T>()`, which convert any `JsonSchema`-derived struct into a formatted field listing including type, default, and description drawn from doc comments.

This ticket wires those two pieces together: add one-line doc comments to every user-facing field in the `Config` struct tree, then replace the `render_config()` stub to call into the help_schema infrastructure and format output grouped by TOML section header.

### Acceptance criteria

- [ ] `apm help config` exits 0 and prints non-empty output to stdout
- [ ] The placeholder string referencing ticket d486d183 no longer appears in `apm help config` output
- [ ] Output contains a recognisable header or path prefix for each top-level section: `project`, `tickets`, `worktrees`, `git_host`, `agents`, `sync`, `logging`, `workers`
- [ ] Output contains a line for `project.name` that is marked as required (no default shown)
- [ ] Output contains a line for `project.default_branch` with default `main`
- [ ] Output contains a line for `agents.max_concurrent` with default `3`
- [ ] Output contains a line for `agents.max_workers_per_epic` with default `1`
- [ ] Output contains a line for `workers.command` with default `claude`
- [ ] Every field line that has a doc comment shows a non-empty description
- [ ] `worker_profiles` appears in the output with a description explaining it is a map of named profiles

### Out of scope

- Content for `render_workflow()`, `render_ticket()`, `render_commands()` — those are sibling tickets 7ba021e8, 14214305, and 3665e017
- ANSI colour or markdown rendering in the output
- Pager integration (no `less`/`more` invocation)
- Per-user-defined `worker_profiles` key documentation — only a generic map description is shown; individual profiles are user-defined and unknowable at build time
- `LocalConfig` and `LocalWorkersOverride` structs — internal override file, not user-facing `config.toml`
- `WorkflowConfig`, `StateConfig`, `TransitionConfig`, and their nested types — covered by ticket 7ba021e8
- `TicketConfig` and `TicketSection` — covered by ticket 14214305
- Changes to the `apm help` dispatcher or topic routing — established by ticket bc89e0a0
- Publishing a JSON Schema file as a build artifact

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:49Z | groomed | in_design | philippepascal |