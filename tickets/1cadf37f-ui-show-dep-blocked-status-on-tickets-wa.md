+++
id = "1cadf37f"
title = "UI: show dep-blocked status on tickets waiting in queue"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "4980"
branch = "ticket/1cadf37f-ui-show-dep-blocked-status-on-tickets-wa"
created_at = "2026-04-02T23:21:21.478449Z"
updated_at = "2026-04-03T22:52:20.708693Z"
+++

## Spec

### Problem

A ticket in `groomed` (or any actionable state) that has unsatisfied `depends_on` deps is silently invisible in the work queue — `apm next` skips it and no worker picks it up. The supervisor board shows the ticket in its swimlane as if it is ready for dispatch, with no visual indication that it is actually dep-blocked.

This creates confusion: the supervisor sees a `groomed` ticket sitting in the column, assumes it will be picked up shortly, and has no immediate way to know it is waiting on another ticket that hasn't reached the required dep gate yet. Diagnosing the stall requires manually running `apm show` and cross-referencing dep states.

The existing lock icon (`Ban` from lucide-react) is rendered at 12px in grey (`text-gray-400`) and blocking details are only available via the browser's native `title` tooltip on hover. This is too subtle — the supervisor needs to see at a glance that a ticket is dep-blocked without hovering, and ideally see which specific tickets are blocking it directly on the card.

The fix is to make dep-blocked status prominently visible on the ticket card: use a coloured background/border treatment to distinguish dep-blocked tickets from actionable ones, and display the blocking ticket IDs and their states directly on the card face rather than hiding them behind a tooltip.

### Acceptance criteria

- [ ] A ticket card with non-empty `blocking_deps` has a visually distinct background or border treatment (not just the existing grey icon) that signals dep-blocked status at a glance
- [ ] A ticket card with empty or absent `blocking_deps` has no dep-blocked visual treatment
- [ ] Each blocking dependency's short ID (first 8 chars) and current state are displayed as text directly on the card face, not just in a hover tooltip
- [ ] The blocking dep IDs on the card are clickable and navigate to the blocking ticket's detail view (call `setSelectedTicketId`)
- [ ] The existing `Ban` icon is replaced or augmented with a more prominent coloured indicator (e.g. amber/orange) when deps are blocking
- [ ] The dep-blocked treatment applies regardless of which swimlane column the ticket appears in (groomed, specd, ready, etc.)
- [ ] When all blocking deps are resolved (ticket refreshes and `blocking_deps` becomes empty), the dep-blocked visual treatment disappears without a page reload

### Out of scope

- Server-side `blocking_deps` computation — already implemented in ticket da95246d
- Adding or editing `depends_on` via the UI
- The ticket detail panel's dep display (already shows blocking deps with click-through)
- The priority queue panel (`PriorityQueuePanel.tsx`) — uses table rows, not cards
- Filtering or sorting swimlanes by dep-blocked status
- Notifications or alerts when a ticket becomes dep-blocked or unblocked

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T23:21Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:27Z | groomed | in_design | philippepascal |
| 2026-04-03T22:47Z | in_design | ready | apm |
| 2026-04-03T22:49Z | ready | in_progress | philippepascal |
| 2026-04-03T22:50Z | in_progress | ammend | apm |
| 2026-04-03T22:52Z | ammend | in_design | philippepascal |