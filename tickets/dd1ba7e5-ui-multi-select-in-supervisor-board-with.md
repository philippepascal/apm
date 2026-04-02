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

- [ ] Shift-clicking a ticket when another ticket in the same column is already focused selects every ticket between them (inclusive) in column render order
- [ ] Clicking a ticket normally (no modifier) clears any existing multi-selection and focuses only that ticket
- [ ] Clicking the column header checkbox when no tickets in the column are selected selects all tickets in that column
- [ ] Clicking the column header checkbox when all tickets in the column are selected deselects all tickets in that column
- [ ] The column header checkbox appears in an indeterminate state when some but not all tickets in the column are selected
- [ ] When 2 or more tickets are selected, the detail panel shows a batch summary listing each selected ticket's 8-char ID, title, and state badge instead of a single ticket's full detail
- [ ] When all selected tickets share at least one common valid transition, that transition appears as a button in the batch summary panel
- [ ] When selected tickets have no common valid transitions, only the priority field appears in the batch summary panel
- [ ] Clicking a batch transition button transitions every selected ticket to the target state
- [ ] The batch priority field applies the entered priority value to every selected ticket on submit
- [ ] Pressing any arrow key while 2 or more tickets are selected clears the multi-selection and moves focus to a single ticket per the existing arrow-key navigation logic

### Out of scope

- Cross-column multi-select (shift-click or header checkbox spanning multiple states)
- Ctrl/Cmd-click for non-contiguous individual toggle
- Drag-to-select
- Batch editing of spec sections or markdown body content
- Batch effort or risk assignment
- Undo/redo for batch operations
- Keyboard shortcut for select-all (Ctrl+A)

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