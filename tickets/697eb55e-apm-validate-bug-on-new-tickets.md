+++
id = "697eb55e"
title = "apm validate bug on new tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/697eb55e-apm-validate-bug-on-new-tickets"
created_at = "2026-06-02T21:18:41.660057Z"
updated_at = "2026-06-03T01:24:34.530805Z"
+++

## Spec

### Problem

`apm validate` runs integrity checks on every non-terminal ticket. One check calls `TicketDocument::validate(&config.ticket.sections)`, which iterates over sections marked `required = true` and flags any that are empty or (for `tasks` sections) contain no checklist items. This check fires regardless of the ticket's current state, so tickets in `new` (and similarly `groomed`, `in_design`, `question`) are flagged even though they haven't been through the spec-writing phase yet. The `required` field's own docstring says it applies "before the ticket can transition out of in_design" — i.e., it is a spec-completeness check, not a universal invariant.

Additionally, the error variant `ValidationError::NoAcceptanceCriteria` hardcodes the string "Acceptance criteria" in its `Display` impl. This means the error message does not reflect the actual section name from the config, violating the principle that validation rules should be derived from config.

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
| 2026-06-02T21:18Z | — | new | philippepascal |
| 2026-06-03T01:24Z | new | groomed | philippepascal |
| 2026-06-03T01:24Z | groomed | in_design | philippepascal |