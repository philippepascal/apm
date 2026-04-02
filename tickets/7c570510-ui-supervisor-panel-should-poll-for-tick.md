+++
id = "7c570510"
title = "UI: supervisor panel should poll for ticket updates to stay fresh"
state = "in_design"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "88876"
branch = "ticket/7c570510-ui-supervisor-panel-should-poll-for-tick"
created_at = "2026-04-02T18:24:16.100111Z"
updated_at = "2026-04-02T19:22:04.537569Z"
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

- WebSocket or server-sent events — HTTP polling is sufficient and consistent with the rest of the UI
- Making the poll interval configurable via UI settings or apm.toml
- Polling for the POST /api/sync (remote fetch) on a background interval — the refresh only re-queries /api/tickets from the local server cache
- Changes to any panel other than SupervisorView (PriorityQueuePanel, WorkEngineControls, WorkerActivityPanel already poll)

### Approach

Single-file change: apm-ui/src/components/supervisor/SupervisorView.tsx

The useQuery call for tickets (currently around line 24) does not pass refetchInterval. Add it:

  const { data: tickets = [], isError: syncError } = useQuery({
    queryKey: ['tickets'],
    queryFn: () => fetch('/api/tickets').then(r => r.json()),
    refetchInterval: 10_000,   // add this line
  })

That is the entire code change. TanStack Query v5 will re-run the queryFn every 10 seconds in the background and update the board reactively. The existing manual sync path (invalidateQueries after POST /api/sync) is unaffected — it triggers an out-of-band immediate refetch on top of the interval.

No backend changes are needed. The GET /api/tickets endpoint is already stateless and cheap (reads from local git refs, no remote fetch).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T18:24Z | — | new | apm |
| 2026-04-02T19:19Z | new | groomed | apm |
| 2026-04-02T19:19Z | groomed | in_design | philippepascal |