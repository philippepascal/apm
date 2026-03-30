+++
id = "5a4ad4bd"
title = "apm work improvement"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "93102"
branch = "ticket/5a4ad4bd-apm-work-improvement"
created_at = "2026-03-30T19:21:34.679718Z"
updated_at = "2026-03-30T19:23:51.068491Z"
+++

## Spec

### Problem

apm work wait for worker it started to finish even if it's using less workers than max. if it's using less worker than max, it should poll regularly in case more tickets have become actionable

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
| 2026-03-30T19:21Z | — | new | apm |
| 2026-03-30T19:23Z | new | in_design | philippepascal |