+++
id = "b8b91642"
title = "make detail panel more reactive when other panels are updated (state of ticket selected might have changed)"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "61636"
branch = "ticket/b8b91642-make-detail-panel-more-reactive-when-oth"
created_at = "2026-04-02T22:28:29.844602Z"
updated_at = "2026-04-02T23:03:08.306716Z"
+++

## Spec

### Problem

The detail panel (TicketDetail.tsx) fetches ticket data via React Query with the key `['ticket', id]` but has no `refetchInterval` configured. This means once it fetches a ticket, it only refreshes when:
- the user explicitly interacts with the detail panel (transitions, patches), or
- the user selects a different ticket and comes back.

Meanwhile the board (SupervisorView) polls `['tickets']` every 10 seconds. When an external agent transitions a ticket's state — moving it from, say, `ready` to `in_progress` — the board card updates within 10 seconds but the detail panel remains stale indefinitely, showing the wrong state badge and a stale set of valid transition buttons.

The same staleness occurs when the user clicks Sync (Shift+S): the sync mutation invalidates `['tickets']` to refresh the board but does not invalidate `['ticket', id]`, so the detail panel still reflects the pre-sync state.

### Acceptance criteria

- [ ] When an external agent transitions the selected ticket's state, the detail panel shows the updated state badge within 15 seconds without any user interaction
- [ ] When the user clicks Sync (Shift+S) and the selected ticket has changed, the detail panel reflects the post-sync state immediately after sync completes
- [ ] The detail panel's transition buttons update to reflect the new valid transitions whenever the ticket state changes remotely
- [ ] The detail panel continues to show stale data correctly during a fetch (no flash to empty/loading state on background refetch)

### Out of scope

- Real-time push (WebSocket / SSE) — polling is sufficient
- Reducing or changing the board's 10-second poll interval
- Invalidating the detail panel when a *different* ticket changes (only the selected ticket matters)
- Cross-browser tab synchronisation

### Approach

Two small changes, both in the frontend:

**1. apm-ui/src/components/TicketDetail.tsx** — add `refetchInterval` to the detail query

```ts
const { data, isLoading, isError, error } = useQuery({
  queryKey: ['ticket', selectedTicketId],
  queryFn: () => fetchTicket(selectedTicketId!),
  enabled: !!selectedTicketId,
  refetchInterval: 10_000,   // ← add this line
})
```

React Query's default behaviour when `refetchInterval` is set is to keep showing the previous data (`staleTime` semantics) while the background refetch is in flight — so there is no flash to a loading skeleton on each poll tick.

**2. apm-ui/src/components/supervisor/SupervisorView.tsx** — also invalidate the detail query family when sync completes

In the `syncMutation.onSuccess` handler, add a second invalidation that matches the `['ticket']` prefix (React Query prefix-matches partial keys):

```ts
onSuccess: () => {
  setSyncError(null)
  queryClient.invalidateQueries({ queryKey: ['tickets'] })
  queryClient.invalidateQueries({ queryKey: ['ticket'] })   // ← add this line
},
```

This covers the explicit Sync flow independently of the polling cadence.

No backend changes needed. No new dependencies.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T22:28Z | — | new | apm |
| 2026-04-02T22:32Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |
| 2026-04-02T22:51Z | in_design | specd | claude-0402-2250-spec1 |
| 2026-04-02T22:55Z | specd | ready | apm |
| 2026-04-02T23:03Z | ready | in_progress | philippepascal |