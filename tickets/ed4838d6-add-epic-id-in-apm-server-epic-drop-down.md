+++
id = "ed4838d6"
title = "add epic id in apm-server epic drop-down"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ed4838d6-add-epic-id-in-apm-server-epic-drop-down"
created_at = "2026-05-28T05:54:59.309955Z"
updated_at = "2026-05-28T06:18:24.295451Z"
+++

## Spec

### Problem

The epic filter dropdown in `apm-server`'s SupervisorView renders each option as `ep.title || ep.id` — showing only the title, or the full raw UUID as fallback. When multiple epics have similar titles, or when the user wants to confirm which epic matches an ID they see elsewhere in the UI (e.g. the 8-char epic chip on TicketCard), the dropdown gives no way to cross-reference.

The TicketCard already displays the first 8 characters of the epic ID as a chip. The dropdown should be consistent: show the 8-char ID prefix alongside the title for every epic option.

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
| 2026-05-28T05:54Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:18Z | groomed | in_design | philippepascal |