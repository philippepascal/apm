+++
id = "2ef15663"
title = "UI: filter the epic bar indicating epics needing refresh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2ef15663-ui-filter-the-epic-bar-indicating-epics-"
created_at = "2026-06-01T17:18:45.877184Z"
updated_at = "2026-06-01T17:18:55.989171Z"
+++

## Spec

### Problem

The Supervisor board has an epic bar — a row of amber/red badges, one per stale epic (branches that are behind `HEAD` and need a `git pull`). When a user applies an epic filter using the dropdown, the ticket swimlanes narrow to only the selected epic's tickets, but the epic bar continues to show all stale epics regardless of the filter. The bar and the board are out of sync.

The desired behaviour: the epic bar should reflect the same epic scope as the rest of the board. When a specific epic filter is active, show only that epic's badge (if it is stale). When the "No epic" filter is active, hide the bar entirely — tickets with no epic cannot belong to a stale epic branch. When no filter is active, the bar behaves exactly as it does today.

### Acceptance criteria

- [ ] When no epic filter is active, the epic bar shows all stale epics that appear in any ticket (existing behaviour unchanged)
- [ ] When a specific epic ID is selected in the epic filter and that epic is stale, the epic bar shows exactly one badge for that epic
- [ ] When a specific epic ID is selected in the epic filter and that epic is not stale, the epic bar is hidden
- [ ] When the "No epic" filter (`__none__`) is active, the epic bar is hidden
- [ ] Stale epics that belong to a different epic than the active filter are not shown in the epic bar while that filter is active

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
| 2026-06-01T17:18Z | — | new | philippepascal |
| 2026-06-01T17:18Z | new | groomed | philippepascal |
| 2026-06-01T17:18Z | groomed | in_design | philippepascal |