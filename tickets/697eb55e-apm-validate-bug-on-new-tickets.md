+++
id = "697eb55e"
title = "apm validate bug on new tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/697eb55e-apm-validate-bug-on-new-tickets"
created_at = "2026-06-02T21:18:41.660057Z"
updated_at = "2026-06-03T01:24:34.530805Z"
+++

## Spec

### Problem

apm validate
error [integrity] : #020ef344 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #e944a5b2 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #b79cd7d1 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #d30843ef [new]: ### Acceptance criteria has no checklist items
error [integrity] : #7ca15981 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #4dae95be [new]: ### Acceptance criteria has no checklist items
error [integrity] : #b45438f8 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #3f59d62c [new]: ### Acceptance criteria has no checklist items
error [integrity] : #472636ab [new]: ### Acceptance criteria has no checklist items
error [integrity] : #9b42371e [new]: ### Acceptance criteria has no checklist items
error [integrity] : #ef26c640 [new]: ### Acceptance criteria has no checklist items
error [integrity] : #4896fdbc [new]: ### Acceptance criteria has no checklist items
error [integrity] : #d3db906e [new]: ### Acceptance criteria has no checklist items
error [integrity] : #cf90954a [new]: ### Acceptance criteria has no checklist items
error [integrity] : #d10b720d [new]: ### Acceptance criteria has no checklist items
error [integrity] : #4f1f2516 [new]: ### Acceptance criteria has no checklist items
22 tickets checked, 0 config errors, 0 warnings, 16 ticket errors
Error: 0 config errors, 16 ticket errors

new tickets do not have to have an acceptance criteria. 
Also make sure that rules on tickets format are derived from config, not hardcoded.

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
| 2026-06-02T21:18Z | — | new | philippepascal |
| 2026-06-03T01:24Z | new | groomed | philippepascal |
| 2026-06-03T01:24Z | groomed | in_design | philippepascal |
