+++
id = "8b3fc691"
title = "apm state: add escape hatch from in_design back to new"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/8b3fc691-apm-state-add-escape-hatch-from-in-desig"
created_at = "2026-03-30T14:44:59.243807Z"
updated_at = "2026-03-30T16:09:11.617095Z"
+++

## Spec

### Problem

if a worker stop/dies while in_design or in_progress, ticket is stuck.
add a --force flag to the apm state command for supervisor only

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
| 2026-03-30T14:44Z | — | new | philippepascal |
| 2026-03-30T16:09Z | new | in_design | philippepascal |
