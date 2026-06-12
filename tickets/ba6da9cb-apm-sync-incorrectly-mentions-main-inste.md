+++
id = "ba6da9cb"
title = "apm sync incorrectly mentions main instead of epic name in error message for missing merge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba6da9cb-apm-sync-incorrectly-mentions-main-inste"
created_at = "2026-06-09T21:47:31.578694Z"
updated_at = "2026-06-12T07:58:00.596811Z"
+++

## Spec

### Problem

when apm sync detects that a ticket hasn't been properly merge despite being marked as implemented, it default the error message to say that the ticket wasn't merged to main.
however the ticket's default branch could be an epic or a default branch. 

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
| 2026-06-09T21:47Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T07:58Z | groomed | in_design | philippepascal |
