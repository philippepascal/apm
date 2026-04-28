+++
id = "14214305"
title = "apm help ticket: render ticket.toml schema from TicketConfig struct"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/14214305-apm-help-ticket-render-ticket-toml-schem"
created_at = "2026-04-28T19:28:31.483927Z"
updated_at = "2026-04-28T19:57:25.528866Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_ticket()` function in `apm/src/cmd/help.rs` — introduced as a stub by ticket bc89e0a0 — returns a placeholder string and does nothing useful. As a result, `apm help ticket` gives users no actionable information about what fields are valid in `.apm/ticket.toml`, their types, defaults, or purpose.

The relevant types (`TicketConfig`, `TicketSection`, `SectionType`) already exist in `apm-core/src/config.rs`. Ticket 069c3403 adds `JsonSchema` derives to those types and supplies `apm_core::help_schema::render_schema::<T>()`, which walks the schema and emits a formatted table of fields with their types, defaults, and doc-comment descriptions. All that is missing is: (1) meaningful doc comments on `TicketConfig`, `TicketSection`, and `SectionType` so `render_schema` has descriptions to emit, and (2) a real body for `render_ticket()` that calls `render_schema::<TicketConfig>()` and prepends a short introductory header.

The `SectionType` enum warrants special attention: its three variants (`free`, `tasks`, `qa`) each have distinct runtime semantics — `tasks` sections integrate with `apm spec --mark` and `apm spec --add-task` — and those semantics should be visible from `apm help ticket` without reading source code.

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
| 2026-04-28T19:28Z | — | new | philippepascal |
| 2026-04-28T19:33Z | new | groomed | philippepascal |
| 2026-04-28T19:57Z | groomed | in_design | philippepascal |