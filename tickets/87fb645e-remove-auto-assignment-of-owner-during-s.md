+++
id = "87fb645e"
title = "Remove auto-assignment of owner during state transitions"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/87fb645e-remove-auto-assignment-of-owner-during-s"
created_at = "2026-04-06T20:57:32.658671Z"
updated_at = "2026-04-06T20:57:32.658671Z"
+++

## Spec

### Problem

When a ticket transitions to in_design (state.rs:113-125) or when work starts via apm start (start.rs:145-201), the owner field is automatically set to the acting agent or user if currently unset. This conflates two separate concerns: who is working on a ticket right now (the agent) and who is responsible for it (the owner). In practice this means agent workers silently claim ownership of tickets they are only implementing, which confuses the supervisor's view of who owns what. Owner assignment should be a deliberate supervisor action — only changeable via explicit commands like 'apm assign' or 'apm set owner', never as a side-effect of state transitions or starting work.

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
| 2026-04-06T20:57Z | — | new | philippepascal |
