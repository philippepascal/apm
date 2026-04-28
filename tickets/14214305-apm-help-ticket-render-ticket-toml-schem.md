+++
id = "14214305"
title = "apm help ticket: render ticket.toml schema from TicketConfig struct"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/14214305-apm-help-ticket-render-ticket-toml-schem"
created_at = "2026-04-28T19:28:31.483927Z"
updated_at = "2026-04-28T19:28:31.483927Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

Replace the `render_ticket()` stub from ticket bc89e0a0 with a real renderer that uses the auto-derive infrastructure from ticket 069c3403 to render the `TicketConfig` and `SectionConfig` structs from `apm-core/src/config.rs` (or wherever they live).

**Structure to render:**
- `[[ticket.sections]]` array — each `SectionConfig` with fields: `name`, `type` (enum: `free`, `tasks`, `qa`), `required`, `placeholder`.
- Top-level explanation that `ticket.sections` is an array defining the spec sections every ticket has, in order.
- Document each section type's semantics:
  - `free` — free-form prose
  - `tasks` — checkbox list (`- [ ] item`); supports `apm spec --mark` and `apm spec --add-task`
  - `qa` — question/answer pairs

**Output structure:**
- Per field: name, type, default, description from doc comments.
- Section-type enum: list variants with semantics (especially the `tasks`/`qa` interactions with `apm spec` flags).

**Implementation pointers:**
- In `apm/src/cmd/help.rs`: replace the stub for `ticket` topic. Call into `apm_core::help_schema` for the `TicketConfig` type.
- Doc comments on `TicketConfig`, `SectionConfig`, and the section-type enum may need to be added or improved as part of this ticket.

**Out of scope:**
- Frontmatter schema (`Frontmatter` struct) — that is the ticket *file* format, not `.apm/ticket.toml`. Could be a follow-up help topic but is not in scope here.
- Examples beyond struct doc comments.

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
