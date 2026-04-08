+++
id = "7c570510"
title = "UI: supervisor panel should poll for ticket updates to stay fresh"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "6365"
branch = "ticket/7c570510-ui-supervisor-panel-should-poll-for-tick"
created_at = "2026-04-02T18:24:16.100111Z"
updated_at = "2026-04-02T20:43:09.679790Z"
+++

## Spec

### Problem

The supervisor panel (SupervisorView.tsx) does not automatically refresh its ticket data. It fetches tickets once on mount and only updates when the user manually triggers a sync via Shift+S (which also calls POST /api/sync to fetch from the remote).

This means that as worker agents transition tickets through states — from ready → in_progress → implemented — the supervisor's kanban board stays frozen on whatever snapshot it loaded at startup. The supervisor has no live view of progress without repeatedly pressing Shift+S.

Every other panel in the UI already polls on a fixed interval: PriorityQueuePanel refreshes every 10 seconds, WorkEngineControls every 3 seconds, WorkerActivityPanel every 5 seconds. The supervisor panel is the odd one out and the most important view for monitoring concurrent agent activity.

### Acceptance criteria

- [x] The supervisor kanban board refreshes its ticket list automatically without any user interaction
- [x] The automatic refresh interval is 10 seconds
- [x] Ticket cards appear in the correct swimlane within 10 seconds of a state transition happening elsewhere
- [x] The manual sync button (Shift+S) continues to work and still triggers a POST /api/sync followed by a data refresh
- [x] No visible flicker or full-board re-render disrupts the user while background polling occurs

### Out of scope

- WebSocket or server-sent events — HTTP polling is sufficient and consistent with the rest of the UI
- Making the poll interval configurable via UI settings or apm.toml
- Polling for the POST /api/sync (remote fetch) on a background interval — the refresh only re-queries /api/tickets from the local server cache
- Changes to any panel other than SupervisorView (PriorityQueuePanel, WorkEngineControls, WorkerActivityPanel already poll)

### Approach

Single-file change: apm-ui/src/components/supervisor/SupervisorView.tsx

Add `refetchInterval: 10_000` to the existing useQuery call — do not change the queryFn or queryKey.

TanStack Query v5 will re-run the queryFn every 10 seconds in the background and update the board reactively. The existing manual sync path (invalidateQueries after POST /api/sync) is unaffected — it triggers an out-of-band immediate refetch on top of the interval.

No backend changes are needed. The GET /api/tickets endpoint is already stateless and cheap (reads from local git refs, no remote fetch).

### Open questions


### Amendment requests

- [x] The Approach snippet shows a hardcoded `fetch('/api/tickets')` queryFn and a plain `queryKey: ['tickets']`. After e8a56566, SupervisorView already uses `queryKey: ['tickets', showClosed]` and a queryFn that appends `?include_closed=true` conditionally. A worker following the snippet literally would regress that feature. Replace the snippet with: "Add `refetchInterval: 10_000` to the existing useQuery call — do not change the queryFn or queryKey."

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T18:24Z | — | new | apm |
| 2026-04-02T19:19Z | new | groomed | apm |
| 2026-04-02T19:19Z | groomed | in_design | philippepascal |
| 2026-04-02T19:22Z | in_design | specd | claude-0402-1930-s7w1 |
| 2026-04-02T20:01Z | specd | ammend | apm |
| 2026-04-02T20:02Z | ammend | in_design | philippepascal |
| 2026-04-02T20:03Z | in_design | specd | claude-0402-2010-x4k2 |
| 2026-04-02T20:06Z | specd | ready | apm |
| 2026-04-02T20:08Z | ready | in_progress | philippepascal |
| 2026-04-02T20:11Z | in_progress | implemented | claude-0402-2010-x4k2 |
| 2026-04-02T20:43Z | implemented | closed | apm-sync |