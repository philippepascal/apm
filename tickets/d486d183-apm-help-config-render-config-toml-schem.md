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
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:49Z | groomed | in_design | philippepascal |