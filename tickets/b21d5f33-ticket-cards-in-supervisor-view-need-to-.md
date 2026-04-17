+++
id = "b21d5f33"
title = "ticket cards in supervisor view need to show epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b21d5f33-ticket-cards-in-supervisor-view-need-to-"
created_at = "2026-04-17T20:17:28.548094Z"
updated_at = "2026-04-17T20:26:52.407984Z"
+++

## Spec

### Problem

Ticket cards in the supervisor board swimlanes do not display which epic a ticket belongs to. The `epic` field is already present in the API response and in the TypeScript `Ticket` type, but `TicketCard.tsx` never renders it. As a result, a supervisor scanning the board has no at-a-glance signal about epic membership — they must open each ticket's detail panel individually to find out.\n\nThe desired behaviour: when a ticket belongs to an epic, the card should show a small, clickable epic badge. Clicking the badge should toggle the board's epic filter to show only tickets in that epic, matching the interaction already implemented in the ticket detail panel.

### Acceptance criteria

- [ ] A ticket card with a non-empty `epic` field shows the first 8 characters of the epic ID as a small monospace badge in the card footer row\n- [ ] A ticket card with no `epic` field shows no epic badge\n- [ ] Clicking the epic badge sets the supervisor board's epic filter to that epic ID\n- [ ] Clicking the epic badge when that epic is already the active filter clears the filter (toggles it off)\n- [ ] Clicking the epic badge does not select or open the ticket (click event is stopped from propagating)

### Out of scope

- Displaying the epic title (only the short 8-char ID is shown)\n- Changes to any other component (detail panel, priority queue panel, filter bar — all already handle epics)\n- Backend API or data model changes (epic is already returned by the tickets endpoint)\n- Epic badge in any non-supervisor view

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T20:17Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:26Z | groomed | in_design | philippepascal |