+++
id = "09fab8d1"
title = "apm-ui: extend dark theme to all columns and fix worker card regressions"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/09fab8d1-apm-ui-extend-dark-theme-to-all-columns-"
created_at = "2026-04-01T06:44:14.497120Z"
updated_at = "2026-04-01T06:44:14.497120Z"
+++

## Spec

### Problem

4d884371 applied a dark theme (bg-gray-900) to the left worker column only. The center (supervisor board) and right (ticket detail) columns still use a light background. The dark theme should be applied consistently across all three columns for a cohesive look.

Additionally, 4d884371 introduced two regressions in WorkerActivityPanel when it replaced the table with a card layout:
1. Worker cards are no longer clickable — clicking a card should select the ticket in the detail panel (setSelectedTicketId), as the table rows did before.
2. The status label ('running' / 'crashed') is gone — replaced by a dot only. The status text should be visible alongside or instead of the dot. The previous table had an explicit status badge that was useful at a glance.

Scope:
- Apply dark background + appropriate text colors to SupervisorView, Swimlane, TicketCard, TicketDetail, and any shared layout containers in WorkScreen
- Restore click-to-select on WorkerActivityPanel cards
- Restore visible status label on WorkerActivityPanel cards (can keep the dot, but add the text)

What is broken or missing, and why it matters.

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
| 2026-04-01T06:44Z | — | new | philippepascal |
