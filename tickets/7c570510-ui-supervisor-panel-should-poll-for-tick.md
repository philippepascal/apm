+++
id = "7c570510"
title = "UI: supervisor panel should poll for ticket updates to stay fresh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "88876"
branch = "ticket/7c570510-ui-supervisor-panel-should-poll-for-tick"
created_at = "2026-04-02T18:24:16.100111Z"
updated_at = "2026-04-02T19:19:30.815976Z"
+++

## Spec

### Problem

The supervisor panel (SupervisorView.tsx) does not automatically refresh its ticket data. It fetches tickets once on mount and only updates when the user manually triggers a sync via Shift+S (which also calls POST /api/sync to fetch from the remote).

This means that as worker agents transition tickets through states — from ready → in_progress → implemented — the supervisor's kanban board stays frozen on whatever snapshot it loaded at startup. The supervisor has no live view of progress without repeatedly pressing Shift+S.

Every other panel in the UI already polls on a fixed interval: PriorityQueuePanel refreshes every 10 seconds, WorkEngineControls every 3 seconds, WorkerActivityPanel every 5 seconds. The supervisor panel is the odd one out and the most important view for monitoring concurrent agent activity.

### Acceptance criteria

- [ ] The supervisor kanban board refreshes its ticket list automatically without any user interaction
- [ ] The automatic refresh interval is 10 seconds
- [ ] Ticket cards appear in the correct swimlane within 10 seconds of a state transition happening elsewhere
- [ ] The manual sync button (Shift+S) continues to work and still triggers a POST /api/sync followed by a data refresh
- [ ] No visible flicker or full-board re-render disrupts the user while background polling occurs

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
| 2026-04-02T18:24Z | — | new | apm |
| 2026-04-02T19:19Z | new | groomed | apm |
| 2026-04-02T19:19Z | groomed | in_design | philippepascal |