+++
id = "c1408201"
title = "clarify roles for setting priority, effort and risk"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c1408201-clarify-roles-for-setting-priority-effor"
created_at = "2026-05-28T05:50:39.594077Z"
updated_at = "2026-05-28T06:16:28.668237Z"
+++

## Spec

### Problem

apm.main-agent.md — add to the grooming flow that setting priority is part of transitioning new → groomed. The supervisor makes the business-value call before handing it to the
   spec-writer.

  apm.spec-writer.md — change the pre-transition block from:
  - apm set <id> effort <1-10>
  - apm set <id> risk <1-10>
  to include priority as a conditional:
  - apm set <id> effort <1-10>
  - apm set <id> risk <1-10>
  - apm set <id> priority <1-10>  — only if not already set by the supervisor
- 
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
| 2026-05-28T05:50Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:16Z | groomed | in_design | philippepascal |
