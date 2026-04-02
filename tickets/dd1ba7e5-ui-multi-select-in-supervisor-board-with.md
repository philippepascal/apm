+++
id = "dd1ba7e5"
title = "UI: multi-select in supervisor board with batch actions"
state = "in_design"
priority = 3
effort = 0
risk = 0
author = "apm"
agent = "31814"
branch = "ticket/dd1ba7e5-ui-multi-select-in-supervisor-board-with"
created_at = "2026-04-02T21:27:15.261676Z"
updated_at = "2026-04-02T22:47:40.101023Z"
+++

## Spec

### Problem

The supervisor board only supports single-ticket selection. Batch operations — grooming a column of `new` tickets, closing a set of `implemented` tickets, setting priority on a group — require clicking each ticket individually. This is tedious and slows routine supervisor housekeeping.

The desired behaviour is:
- **Shift-click** a second ticket to select a contiguous range (all tickets between the first and second selection within the same column, in render order)
- **Column header checkbox** to select or deselect all tickets in that column at once

When 2+ tickets are selected the detail panel switches to a batch summary view that lists the selected ticket IDs, titles, and states, and offers only actions that are valid for every selected ticket (common state transitions and batch priority adjustment).

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
| 2026-04-02T21:27Z | — | new | apm |
| 2026-04-02T22:46Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |