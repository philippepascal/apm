+++
id = "ffaad988"
title = "apm start and apm state: set and clear owner on transitions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/ffaad988-apm-start-and-apm-state-set-and-clear-ow"
created_at = "2026-04-04T06:28:06.049762Z"
updated_at = "2026-04-04T06:46:10.393369Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

Once an `owner` field exists on tickets, it needs to be set at the right moment. Today `apm start` writes the agent name to the History section but nothing in frontmatter. The owner should persist for the entire ticket lifecycle — once someone owns a ticket, they own it through design, implementation, review, and completion. Ownership is only transferred by explicit supervisor action (`apm assign`), never cleared automatically on state transitions. `apm start` and `apm state in_design` should set the owner when claiming an unowned ticket. If the ticket already has an owner, these commands should still work (the same person resuming work) but not silently overwrite a different owner — that requires `apm assign`.

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
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:46Z | groomed | in_design | philippepascal |
