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

- [ ] The SupervisorView toolbar shows a "New epic" button next to the existing "New ticket" button
- [ ] Clicking "New epic" opens a modal with a title input field
- [ ] Submitting the modal with a non-empty title sends POST /api/epics and closes the modal on success
- [ ] After successful creation the new epic appears in the SupervisorView epic-filter dropdown without a page refresh
- [ ] Submitting the modal with an empty title shows a validation error and does not send a request
- [ ] The modal can be dismissed by pressing Escape
- [ ] The modal can be dismissed by clicking the backdrop

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