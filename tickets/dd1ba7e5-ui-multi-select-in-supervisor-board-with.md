+++
id = "dd1ba7e5"
title = "UI: multi-select in supervisor board with batch actions"
state = "groomed"
priority = 3
effort = 0
risk = 0
author = "apm"
branch = "ticket/dd1ba7e5-ui-multi-select-in-supervisor-board-with"
created_at = "2026-04-02T21:27:15.261676Z"
updated_at = "2026-04-02T22:46:58.026773Z"
+++

## Spec

### Problem

The supervisor board only supports single-ticket selection. Batch operations — grooming a column of new tickets, closing a set of implemented tickets, setting priority on a group — require clicking each ticket individually. This is tedious and slows down routine supervisor housekeeping.

Multi-select should work via:
- **Shift-click** a second ticket to select a range (all tickets between the first and second selection within the same column)
- **Click a column header checkbox** to select all tickets in that column

When multiple tickets are selected:
- The detail panel switches to a summary view showing the selected ticket IDs, titles, and states
- Only actions valid for **all** selected tickets are offered (e.g. if all are `specd`, "approve" and "request changes" appear; if states are mixed, only state-agnostic actions like "set priority" or "close" appear where applicable)
- Keyboard navigation (arrow keys) clears the multi-selection and moves to a single ticket

This enables batch grooming, batch state transitions (e.g. close a column of stale tickets), and bulk priority adjustment without needing a separate admin flow.

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
