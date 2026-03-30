+++
id = "5ec960eb"
title = "refactor: move spec section logic from spec.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "philippepascal"
branch = "ticket/5ec960eb-refactor-move-spec-section-logic-from-sp"
created_at = "2026-03-30T14:27:31.109323Z"
updated_at = "2026-03-30T16:31:26.876353Z"
+++

## Spec

### Problem

`spec.rs` contains 394 lines of spec document manipulation logic that belongs in
`apm-core`:

- Section name validation against config-defined sections
- Section format validation (tasks/checkboxes, QA format, free text)
- Document parsing and serialization (section get/set by heading)
- Acceptance criteria checkbox parsing and toggling (`mark` subcommand)
- Required section presence validation
- Body manipulation utilities shared with other commands but duplicated

This logic is not CLI-specific — it operates on ticket document structure.
`apm-serve` will need to read and write spec sections from the browser (e.g.
checking off acceptance criteria). Without this refactor it must shell out to
`apm spec` or duplicate all the parsing.

Target: `apm_core::spec` module exposing `get_section()`, `set_section()`,
`mark_item()`, `validate_sections()`. CLI `spec.rs` becomes a thin wrapper
of ~50 lines.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
