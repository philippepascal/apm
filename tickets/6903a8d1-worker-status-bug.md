+++
id = "6903a8d1"
title = "worker status bug"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/6903a8d1-worker-status-bug"
created_at = "2026-04-04T16:07:08.053019Z"
updated_at = "2026-04-04T16:39:42.876351Z"
+++

## Spec

### Problem

based on the current config, only workers in "in_design" or "in_progress" can have a "crashed" status if their pid is not running.
if we are missing a parameter on the state to figure that out, propose one. "instructions" could imply it.

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
| 2026-04-04T16:07Z | — | new | philippepascal |
| 2026-04-04T16:39Z | new | groomed | apm |
