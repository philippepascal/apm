+++
id = "305281a7"
title = "UI supervisor panel ticket don't show if they are part of epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "51125"
branch = "ticket/305281a7-ui-supervisor-panel-ticket-don-t-show-if"
created_at = "2026-04-02T22:32:22.237758Z"
updated_at = "2026-04-02T22:48:01.338860Z"
+++

## Spec

### Problem

The supervisor board currently shows every ticket regardless of whether it belongs to an epic. This creates visual noise: a ticket already tracked under an epic appears twice — once inside the epic and once in the top-level board. As the number of epics and their child tickets grows, the board becomes cluttered and harder to scan.

Tickets that belong to an epic are managed at the epic level. For the supervisor's top-level view the right unit of work is the epic, not each constituent ticket. Epic-member tickets should therefore be hidden from the default board view, with an opt-in toggle to reveal them when needed (e.g. to inspect all blocked tickets across every epic).

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
| 2026-04-02T22:32Z | — | new | apm |
| 2026-04-02T22:32Z | new | groomed | apm |
| 2026-04-02T22:48Z | groomed | in_design | philippepascal |