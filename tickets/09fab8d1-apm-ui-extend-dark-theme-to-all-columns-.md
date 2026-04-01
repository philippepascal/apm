+++
id = "09fab8d1"
title = "apm-ui: extend dark theme to all columns and fix worker card regressions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "65373"
branch = "ticket/09fab8d1-apm-ui-extend-dark-theme-to-all-columns-"
created_at = "2026-04-01T06:44:14.497120Z"
updated_at = "2026-04-01T06:44:57.458950Z"
+++

## Spec

### Problem

Commit 4d884371 applied a dark background (bg-gray-900) only to the left WorkerView column. The center column (SupervisorView) still uses bg-gray-50 and the right column (TicketDetail) uses bg-white, making the three-column layout visually inconsistent. The dark theme must be applied uniformly so all columns share the same dark palette.

The same commit replaced WorkerActivityPanel's table with a card layout but introduced two regressions:
1. Click-to-select removed. The old table rows called setSelectedTicketId on click; the new card divs have no onClick handler, so clicking a worker card no longer opens that ticket in the detail panel.
2. Status label removed. The old table had an explicit text badge ('running' / 'crashed'). The new cards show only a green or red dot with no label, making status harder to read at a glance.

These regressions affect every user of the UI who relies on clicking a worker to jump to its ticket, and on reading worker status without hovering.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:44Z | — | new | philippepascal |
| 2026-04-01T06:44Z | new | in_design | philippepascal |