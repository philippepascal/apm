+++
id = "7f61c54a"
title = "apm-ui: priority reorder via drag-and-drop in worker queue"
state = "closed"
priority = 35
effort = 5
risk = 3
author = "apm"
agent = "32939"
branch = "ticket/7f61c54a-apm-ui-priority-reorder-via-drag-and-dro"
created_at = "2026-03-31T06:13:11.256058Z"
updated_at = "2026-04-01T06:21:06.603216Z"
+++

## Spec

### Problem

The priority queue panel in the bottom half of the left column (introduced by Step 7b) is read-only. Users have no way to influence the dispatch order from the UI — apm next always picks based on computed score — without reaching for the CLI. This means the UI does not fulfil the core supervisor workflow of adjusting which ticket gets worked on next.

The desired behaviour: dragging a ticket card up or down in the queue, or pressing up/down keyboard shortcuts while a queue item is focused, reorders the queue and persists the new order by updating the ticket priority frontmatter field via PATCH /api/tickets/:id. Because the apm next score formula is priority * priority_weight + effort * effort_weight + risk * risk_weight, updating priority is sufficient to control relative dispatch order when priority values are spread far enough apart.

Affected users: anyone supervising an apm work session via the web UI.

### Acceptance criteria

- [x] Dragging a ticket card to a new position in the queue reorders the list visually before the server responds (optimistic update)
- [x] After a drag reorder, PATCH /api/tickets/:id is sent with the updated priority value for the moved ticket
- [x] The queue order after drag reflects the same dispatch order that apm next would compute
- [x] Pressing the up-arrow key while a queue item is focused moves it one position up in the queue
- [x] Pressing the down-arrow key while a queue item is focused moves it one position down in the queue
- [x] Keyboard reorder triggers the same PATCH /api/tickets/:id call as drag reorder
- [x] If the PATCH request fails, the queue reverts to its pre-reorder order and an error toast is shown
- [x] Reordering the priority queue does not affect the swimlane layout in the middle column
- [x] A ticket with state in_progress cannot be reordered in the queue via drag or keyboard
- [x] PATCH /api/tickets/:id accepts a JSON body with a priority integer (0-255) and persists it to the ticket branch via git

### Out of scope

- Priority editing via the inline click-to-edit field in the ticket detail panel (covered by Step 13b)
- Reordering tickets in the supervisor swimlanes (middle column)
- Batch priority normalization across the entire ticket set
- Drag-and-drop between the priority queue and other panels
- Touch / mobile drag support
- Persisting visual order independently of the priority field (no separate ordering index; priority is the single source of truth)

### Approach

**Backend: PATCH /api/tickets/:id**

Add a PATCH handler to the axum router (in the apm-server crate, alongside the existing GET /api/tickets routes from Step 2). The handler:
1. Deserialises the JSON body into a struct with an optional `priority: Option<u8>` field (other patchable fields may be added later by Step 13b)
2. Calls `ticket::set_field(&mut fm, "priority", &value.to_string())` from apm-core, identical to what `apm set` does
3. Calls `git::commit_to_branch` to persist the change to the ticket branch
4. Returns the updated ticket as JSON (same shape as GET /api/tickets/:id)
5. If the priority value is out of range (>255), return HTTP 422 with an error message

This reuses existing apm-core logic — no new core functions are needed.

**Frontend: drag-and-drop**

Use `@dnd-kit/core` and `@dnd-kit/sortable` for drag-and-drop. These libraries are pointer-event based, work with React, and are compatible with shadcn/ui without patching the DOM.

In the priority queue panel component (from Step 7b):
1. Wrap the ticket list in `<SortableContext items={...} strategy={verticalListSortingStrategy}>`
2. Each ticket card becomes a `<SortableItem>` with `useSortable(id)`
3. Wrap the panel in `<DndContext onDragEnd={handleDragEnd}>`

`handleDragEnd` logic:
1. Determine the new index of the dragged item
2. Compute the new priority value: assign priorities so the queue order matches `apm next` ordering. Strategy: after any reorder, assign priorities to ALL queue items based on their new visual position — item at index 0 gets `baseCount + (n-1)`, index 1 gets `baseCount + (n-2)`, etc., where `baseCount` keeps values away from zero to leave headroom. Use step 10 (e.g. positions get 10, 20, 30, ..., reversed) so a single drag only truly needs to update the moved ticket, but to avoid integer collisions send updates for all affected tickets.
3. Optimistically update the Zustand store immediately
4. Fire PATCH requests (one per changed ticket) via TanStack Query mutations
5. On any mutation error: revert to the pre-drag snapshot stored before `handleDragEnd` and show a toast

**Frontend: keyboard shortcuts**

When a queue item has focus, the up/down arrow keys call the same reorder function as drag. Guard: do not propagate the event to the global arrow-key navigation handler (which moves ticket selection) — consume the event with `stopPropagation` only when the queue item is focused and the list has more than one item.

**Disabled state for in_progress tickets**

In `handleDragEnd` and the keyboard handler, skip any operation if the ticket's state is `in_progress`. The drag handle should also be visually disabled (reduced opacity, no-grab cursor) for in_progress tickets.

**Files changed**
- `apm-server/src/routes/tickets.rs` — add PATCH handler and request/response types
- `apm-ui/src/components/PriorityQueue.tsx` — add DnD context, sortable wrappers, keyboard handling, optimistic updates
- `apm-ui/package.json` — add `@dnd-kit/core` and `@dnd-kit/sortable` dependencies

**Order of steps**
1. Add and test PATCH /api/tickets/:id backend endpoint
2. Install dnd-kit packages
3. Wire DnD into PriorityQueue component with optimistic update
4. Add keyboard up/down handling
5. Add disabled state for in_progress tickets

### Open questions



### Amendment requests

- [x] Fix typo in File Changes Summary: `apm-serve/src/routes/tickets.rs` → `apm-server/src/routes/tickets.rs`

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T06:57Z | new | in_design | philippepascal |
| 2026-03-31T07:01Z | in_design | specd | claude-0331-0657-4370 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:13Z | ammend | in_design | philippepascal |
| 2026-03-31T19:17Z | in_design | specd | claude-0331-1913-b2c4 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T06:01Z | ready | in_progress | philippepascal |
| 2026-04-01T06:13Z | in_progress | implemented | claude-0401-0602-c8e0 |
| 2026-04-01T06:20Z | implemented | accepted | apm |
| 2026-04-01T06:21Z | accepted | closed | apm-sync |