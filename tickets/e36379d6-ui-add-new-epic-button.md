+++
id = "e36379d6"
title = "UI: add new epic button"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "26175"
branch = "ticket/e36379d6-ui-add-new-epic-button"
created_at = "2026-04-02T20:47:05.242823Z"
updated_at = "2026-04-02T20:56:20.210761Z"
+++

## Spec

### Problem

The web UI provides no way to create epics. The only paths to epic creation are the CLI (`apm epic new`) and direct API calls. The SupervisorView toolbar already has a "New ticket" button that opens a modal, but there is no parallel affordance for epics, forcing supervisors to drop out of the UI whenever they need to define a new epic grouping.

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
| 2026-04-02T20:47Z | — | new | apm |
| 2026-04-02T20:50Z | new | groomed | apm |
| 2026-04-02T20:56Z | groomed | in_design | philippepascal |