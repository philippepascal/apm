+++
id = "b21d5f33"
title = "ticket cards in supervisor view need to show epic"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b21d5f33-ticket-cards-in-supervisor-view-need-to-"
created_at = "2026-04-17T20:17:28.548094Z"
updated_at = "2026-04-18T01:02:56.733030Z"
+++

## Spec

### Problem

Ticket cards in the supervisor board swimlanes do not display which epic a ticket belongs to. The `epic` field is already present in the API response and in the TypeScript `Ticket` type, but `TicketCard.tsx` never renders it. As a result, a supervisor scanning the board has no at-a-glance signal about epic membership — they must open each ticket's detail panel individually to find out.\n\nThe desired behaviour: when a ticket belongs to an epic, the card should show a small, clickable epic badge. Clicking the badge should toggle the board's epic filter to show only tickets in that epic, matching the interaction already implemented in the ticket detail panel.

### Acceptance criteria

- [x] A ticket card with a non-empty `epic` field shows the first 8 characters of the epic ID as a small monospace badge in the card footer row\n- [x] A ticket card with no `epic` field shows no epic badge\n- [x] Clicking the epic badge sets the supervisor board's epic filter to that epic ID\n- [x] Clicking the epic badge when that epic is already the active filter clears the filter (toggles it off)\n- [x] Clicking the epic badge does not select or open the ticket (click event is stopped from propagating)

### Out of scope

- Displaying the epic title (only the short 8-char ID is shown)\n- Changes to any other component (detail panel, priority queue panel, filter bar — all already handle epics)\n- Backend API or data model changes (epic is already returned by the tickets endpoint)\n- Epic badge in any non-supervisor view

### Approach

Single file change: `apm-ui/src/components/supervisor/TicketCard.tsx`.\n\n1. Destructure `epicFilter` and `setEpicFilter` from the `useLayoutStore` call on line 12 (alongside the existing destructured values).\n\n2. In the footer row `div` (currently at line 98, which renders the short ticket ID and owner), insert a conditional epic badge between the ticket-ID `span` and the owner `span`:\n\n```tsx\n{ticket.epic && (\n  <button\n    onClick={e => { e.stopPropagation(); setEpicFilter(epicFilter === ticket.epic ? null : ticket.epic!) }}\n    title={`Epic: ${ticket.epic}`}\n    className={\n      'text-[10px] font-mono px-1 rounded border ' +\n      (epicFilter === ticket.epic\n        ? 'border-blue-500 text-blue-300 bg-blue-900/30'\n        : 'border-gray-600 text-gray-500 hover:text-gray-300')\n    }\n  >\n    {ticket.epic.slice(0, 8)}\n  </button>\n)}\n```\n\nNo other files require changes. The `Ticket` type in `types.ts` already declares `epic?: string`, and the API already returns the field.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T20:17Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:26Z | groomed | in_design | philippepascal |
| 2026-04-17T20:29Z | in_design | specd | claude-0417-2026-d5d0 |
| 2026-04-17T21:45Z | specd | ready | apm |
| 2026-04-17T21:48Z | ready | in_progress | philippepascal |
| 2026-04-17T21:50Z | in_progress | implemented | claude-0417-2148-8d60 |
| 2026-04-18T01:02Z | implemented | closed | philippepascal |
