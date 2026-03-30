+++
id = "5ec960eb"
title = "refactor: move spec section logic from spec.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "99956"
branch = "ticket/5ec960eb-refactor-move-spec-section-logic-from-sp"
created_at = "2026-03-30T14:27:31.109323Z"
updated_at = "2026-03-30T16:31:26.876353Z"
+++

## Spec

### Problem

spec.rs contains 394 lines of spec-document manipulation logic that belongs in apm-core:

- Section format enforcement based on SectionType (tasks/checkboxes, QA format, free text)
- Setting named sections on TicketDocument fields (Problem, Acceptance criteria, Out of scope, Approach, Open questions, Amendment requests)
- Acceptance criteria checkbox parsing and toggling (the `mark` subcommand)
- Raw body section get/set for custom sections not mapped to TicketDocument fields
- Section printing helpers shared by multiple code paths

None of this is CLI-specific — it all operates on ticket document structure.
apm-serve will need to read and write spec sections from the browser (e.g.
checking off acceptance criteria). Without this refactor it must shell out to
`apm spec` or duplicate all the parsing logic.

Target: a new `apm_core::spec` module exposing `get_section()`, `set_section()`,
`apply_section_type()`, `mark_item()`, `get_section_body()`, and `set_section_body()`.
CLI `spec.rs` becomes a thin wrapper of ~50 lines.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |