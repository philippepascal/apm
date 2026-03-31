+++
id = "7f61c54a"
title = "apm-ui: priority reorder via drag-and-drop in worker queue"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "58553"
branch = "ticket/7f61c54a-apm-ui-priority-reorder-via-drag-and-dro"
created_at = "2026-03-31T06:13:11.256058Z"
updated_at = "2026-03-31T06:57:21.975360Z"
+++

## Spec

### Problem

The priority queue panel in the bottom half of the left column (introduced by Step 7b) is read-only. Users have no way to influence the dispatch order from the UI — apm next always picks based on computed score — without reaching for the CLI. This means the UI does not fulfil the core supervisor workflow of adjusting which ticket gets worked on next.

The desired behaviour: dragging a ticket card up or down in the queue, or pressing up/down keyboard shortcuts while a queue item is focused, reorders the queue and persists the new order by updating the ticket priority frontmatter field via PATCH /api/tickets/:id. Because the apm next score formula is priority * priority_weight + effort * effort_weight + risk * risk_weight, updating priority is sufficient to control relative dispatch order when priority values are spread far enough apart.

Affected users: anyone supervising an apm work session via the web UI.

### Acceptance criteria

- [ ] Dragging a ticket card to a new position in the queue reorders the list visually before the server responds (optimistic update)
- [ ] After a drag reorder, PATCH /api/tickets/:id is sent with the updated priority value for the moved ticket
- [ ] The queue order after drag reflects the same dispatch order that apm next would compute
- [ ] Pressing the up-arrow key while a queue item is focused moves it one position up in the queue
- [ ] Pressing the down-arrow key while a queue item is focused moves it one position down in the queue
- [ ] Keyboard reorder triggers the same PATCH /api/tickets/:id call as drag reorder
- [ ] If the PATCH request fails, the queue reverts to its pre-reorder order and an error toast is shown
- [ ] Reordering the priority queue does not affect the swimlane layout in the middle column
- [ ] A ticket with state in_progress cannot be reordered in the queue via drag or keyboard
- [ ] PATCH /api/tickets/:id accepts a JSON body with a priority integer (0-255) and persists it to the ticket branch via git

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T06:57Z | new | in_design | philippepascal |