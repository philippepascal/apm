+++
id = "1cadf37f"
title = "UI: show dep-blocked status on tickets waiting in queue"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/1cadf37f-ui-show-dep-blocked-status-on-tickets-wa"
created_at = "2026-04-02T23:21:21.478449Z"
updated_at = "2026-04-04T06:00:54.093461Z"
+++

## Spec

### Problem

A ticket in `groomed` (or any actionable state) that has unsatisfied `depends_on` deps is silently invisible in the work queue — `apm next` skips it and no worker picks it up. The supervisor board shows the ticket in its swimlane as if it is ready for dispatch, with no visual indication that it is actually dep-blocked.

This creates confusion: the supervisor sees a `groomed` ticket sitting in the column, assumes it will be picked up shortly, and has no immediate way to know it is waiting on another ticket that hasn't reached the required dep gate yet. Diagnosing the stall requires manually running `apm show` and cross-referencing dep states.

The existing lock icon (`Ban` from lucide-react) is rendered at 12px in grey (`text-gray-400`) and blocking details are only available via the browser's native `title` tooltip on hover. This is too subtle — the supervisor needs to see at a glance that a ticket is dep-blocked without hovering, and ideally see which specific tickets are blocking it directly on the card.

The fix is to make dep-blocked status prominently visible on the ticket card: use a coloured background/border treatment to distinguish dep-blocked tickets from actionable ones, and display the blocking ticket IDs and their states directly on the card face rather than hiding them behind a tooltip.

### Acceptance criteria

- [x] A ticket card with non-empty `blocking_deps` has a visually distinct background or border treatment (not just the existing grey icon) that signals dep-blocked status at a glance
- [x] A ticket card with empty or absent `blocking_deps` has no dep-blocked visual treatment
- [x] Each blocking dependency's short ID (first 8 chars) and current state are displayed as text directly on the card face, not just in a hover tooltip
- [x] The blocking dep IDs on the card are clickable and navigate to the blocking ticket's detail view (call `setSelectedTicketId`)
- [x] The existing `Ban` icon is replaced or augmented with a more prominent coloured indicator (e.g. amber/orange) when deps are blocking
- [x] The dep-blocked treatment applies regardless of which swimlane column the ticket appears in (groomed, specd, ready, etc.)
- [x] When all blocking deps are resolved (ticket refreshes and `blocking_deps` becomes empty), the dep-blocked visual treatment disappears without a page reload

### Out of scope

- Server-side `blocking_deps` computation — already implemented in ticket da95246d
- Adding or editing `depends_on` via the UI
- The ticket detail panel's dep display (already shows blocking deps with click-through)
- The priority queue panel (`PriorityQueuePanel.tsx`) — uses table rows, not cards
- Filtering or sorting swimlanes by dep-blocked status
- Notifications or alerts when a ticket becomes dep-blocked or unblocked

### Approach

All changes are in a single file: `apm-ui/src/components/supervisor/TicketCard.tsx`. No server or type changes needed — `blocking_deps` is already computed server-side and typed in `types.ts`.

**1. Card-level dep-blocked styling**

Add a conditional class to the outermost `<div>` of `TicketCard`. When `ticket.blocking_deps?.length` is truthy, apply a border accent and slightly different background to make dep-blocked cards visually distinct:

- Non-blocked cards keep existing: `border-gray-600 bg-gray-800 hover:bg-gray-700`
- Dep-blocked cards get: `border-amber-700/60 bg-amber-950/20 hover:bg-amber-950/30`

Extract an `isDepBlocked` boolean at the top of the component and use it in the className expression. The amber tint stands out against the grey board without being garish. Selection ring classes (`ring-2 ring-blue-400` / `ring-2 ring-blue-500`) remain unchanged and layer on top.

**2. Replace grey Ban icon with coloured indicator**

Change the existing `Ban` icon's `className` from `text-gray-400` to `text-amber-400` so the icon colour matches the card accent.

**3. Show blocking dep details on the card face**

Below the title `<p>` and above the bottom ID/agent row, add a new section that renders when deps are blocking. For each entry in `blocking_deps`, render a small amber pill button showing `<short-id>: <state>`:

- Use `text-[10px] font-mono px-1 rounded bg-amber-900/40 text-amber-300 hover:bg-amber-800/50`
- On click, call `e.stopPropagation()` then `setSelectedTicketId(dep.id)` to navigate to the blocking ticket
- Wrap in a flex container with `gap-1 mt-1`

This pattern matches the dep display in `TicketDetail.tsx` (line 386) where dep IDs are clickable buttons that navigate to the dep's detail view.

**4. Wire `setSelectedTicketId` into the component**

`setSelectedTicketId` is already destructured from `useLayoutStore()` on line 11, so no additional wiring is needed.

**Reactivity:** The ticket list auto-refreshes via react-query polling. When blocking deps are resolved server-side, the next poll returns empty `blocking_deps` and the amber treatment disappears automatically — no special handling needed.

**No tests needed:** This is a purely cosmetic UI change in a React component. The server-side `blocking_deps` computation is already tested (see `list_tickets_blocking_deps` test in `apm-server/src/main.rs`). The UI change is best verified visually on the supervisor board.

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
| 2026-04-03T22:55Z | in_design | specd | claude-0403-2255-b7c1 |
| 2026-04-04T00:30Z | specd | ready | apm |
| 2026-04-04T01:55Z | ready | in_progress | philippepascal |
| 2026-04-04T01:57Z | in_progress | implemented | claude-0404-0155-9ce0 |
| 2026-04-04T06:00Z | implemented | closed | apm-sync |
