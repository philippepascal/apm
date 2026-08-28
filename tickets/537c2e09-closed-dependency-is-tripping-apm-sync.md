+++
id = "537c2e09"
title = "closed dependency is tripping apm sync"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/537c2e09-closed-dependency-is-tripping-apm-sync"
created_at = "2026-08-28T00:48:10.874316Z"
updated_at = "2026-08-28T07:13:07.476822Z"
+++

## Spec

### Problem

apm sync
  #cfd8d425: dep 8f85c68c not found
error: config has changed and validation is failing.
Mutating commands are blocked. Run apm validate to fix.

in this example 8f85c68c is closed and clean, however the ticket can be find in the tickets directory.

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
| 2026-08-28T00:48Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
