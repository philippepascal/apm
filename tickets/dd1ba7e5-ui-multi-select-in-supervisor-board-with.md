+++
id = "dd1ba7e5"
title = "UI: multi-select in supervisor board with batch actions"
state = "ammend"
priority = 3
effort = 5
risk = 3
author = "apm"
agent = "25392"
branch = "ticket/dd1ba7e5-ui-multi-select-in-supervisor-board-with"
created_at = "2026-04-02T21:27:15.261676Z"
updated_at = "2026-04-02T23:10:04.310724Z"
+++

## Spec

### Problem

The supervisor board only supports single-ticket selection. Batch operations — grooming a column of `new` tickets, closing a set of `implemented` tickets, setting priority on a group — require clicking each ticket individually. This is tedious and slows routine supervisor housekeeping.

The desired behaviour is:
- **Shift-click** a second ticket to select a contiguous range (all tickets between the first and second selection within the same column, in render order)
- **Column header checkbox** to select or deselect all tickets in that column at once

When 2+ tickets are selected the detail panel switches to a batch summary view that lists the selected ticket IDs, titles, and states, and offers only actions that are valid for every selected ticket (common state transitions and batch priority adjustment).

### Acceptance criteria

- [x] Shift-clicking a ticket when another ticket in the same column is already focused selects every ticket between them (inclusive) in column render order
- [x] Clicking a ticket normally (no modifier) clears any existing multi-selection and focuses only that ticket
- [x] Clicking the column header checkbox when no tickets in the column are selected selects all tickets in that column
- [x] Clicking the column header checkbox when all tickets in the column are selected deselects all tickets in that column
- [x] The column header checkbox appears in an indeterminate state when some but not all tickets in the column are selected
- [x] When 2 or more tickets are selected, the detail panel shows a batch summary listing each selected ticket's 8-char ID, title, and state badge instead of a single ticket's full detail
- [x] When all selected tickets share at least one common valid transition, that transition appears as a button in the batch summary panel
- [x] When selected tickets have no common valid transitions, only the priority field appears in the batch summary panel
- [x] Clicking a batch transition button transitions every selected ticket to the target state
- [x] The batch priority field applies the entered priority value to every selected ticket on submit
- [x] Pressing any arrow key while 2 or more tickets are selected clears the multi-selection and moves focus to a single ticket per the existing arrow-key navigation logic

### Out of scope

- Cross-column multi-select (shift-click or header checkbox spanning multiple states)
- Ctrl/Cmd-click for non-contiguous individual toggle
- Drag-to-select
- Batch editing of spec sections or markdown body content
- Batch effort or risk assignment
- Undo/redo for batch operations
- Keyboard shortcut for select-all (Ctrl+A)

### Approach

**1. `apm-ui/src/store/useLayoutStore.ts`**

Add to the store:
- `selectedTicketIds: string[]` — the current multi-select set (empty when none)
- `lastClickedTicketId: string | null` — anchor for shift-click range
- `selectTicketRange(ids: string[])` — replace `selectedTicketIds` with the given ordered slice and set `lastClickedTicketId` to the last element
- `selectColumn(ids: string[])` — set `selectedTicketIds` to all IDs in the column
- `deselectColumn(ids: string[])` — remove all column IDs from `selectedTicketIds`
- `clearMultiSelection()` — set `selectedTicketIds = []`

Update `setSelectedTicketId(id)` to also clear `selectedTicketIds` and set `lastClickedTicketId = id`. This keeps single-select backward-compatible.

**2. `apm-ui/src/components/supervisor/TicketCard.tsx`**

- Read `selectedTicketIds` and `lastClickedTicketId` from the store
- Accept a new `columnTicketIds: string[]` prop (ordered IDs of all tickets in the same column, passed by `Swimlane`)
- On `onClick`:
  - If `event.shiftKey` and `lastClickedTicketId` is present in `columnTicketIds`: compute the range slice between `lastClickedTicketId` and `ticket.id`, call `selectTicketRange(slice)`
  - Otherwise: call `setSelectedTicketId(ticket.id)` (clears multi-select per item 1 above)
- Derive `isMultiSelected = selectedTicketIds.includes(ticket.id)`
- Apply a distinct visual (e.g. `ring-2 ring-blue-400 bg-gray-700/60`) for multi-selected cards alongside the existing single-select ring

**3. `apm-ui/src/components/supervisor/Swimlane.tsx`**

- Read `selectedTicketIds`, `selectColumn`, `deselectColumn` from store
- Compute:
  - `allSelected = tickets.length > 0 && tickets.every(t => selectedTicketIds.includes(t.id))`
  - `someSelected = tickets.some(t => selectedTicketIds.includes(t.id))`
- Add a `<input type="checkbox">` to the column header:
  - `checked={allSelected}`, `ref` callback sets `indeterminate` when `someSelected && !allSelected`
  - `onChange`: if `allSelected` → `deselectColumn(ids)`, else → `selectColumn(ids)`
- Pass `columnTicketIds={tickets.map(t => t.id)}` to each `TicketCard`

**4. `apm-ui/src/components/TicketDetail.tsx`**

- Read `selectedTicketIds` from store
- If `selectedTicketIds.length > 1`, render `<BatchDetailPanel ids={selectedTicketIds} />` instead of the existing single-ticket content

`BatchDetailPanel` (new component, same file or a separate `BatchDetailPanel.tsx`):
- Use `useQueries` to fetch `/api/tickets/{id}` for each selected ID in parallel
- While loading: show a spinner / "Loading N tickets…"
- Render a scrollable list of rows: `[id.slice(0,8)] title — state`
- Compute `commonTransitions`: intersection of `valid_transitions` arrays across all loaded tickets (match on `to` field)
- Render one button per common transition (same style as existing `TransitionButtons`)
- On transition click: `POST /api/tickets/batch/transition` with `{ ids, to }` body; on success invalidate `['tickets']` query and clear multi-selection
- Render a priority `InlineNumberField` that on submit `POST /api/tickets/batch/priority` with `{ ids, priority }`; on success invalidate `['tickets']`

**5. `apm-server/src/main.rs`**

Add two new route handlers:

`POST /api/tickets/batch/transition`
- Body: `{ ids: Vec<String>, to: String }`
- For each id: call `apm_core::state::transition(...)` in a blocking task
- Return `200 { succeeded: Vec<String>, failed: Vec<{ id: String, error: String }> }`
- Register route alongside the existing `/api/tickets/:id/transition` route

`POST /api/tickets/batch/priority`  
- Body: `{ ids: Vec<String>, priority: i64 }`
- For each id: PATCH priority field (reuse or call the same logic as the existing PATCH `/api/tickets/:id` handler)
- Return same `succeeded/failed` shape

**6. `apm-ui/src/components/WorkScreen.tsx`**

In the arrow-key `handleKeyDown` handler (around line 74–100): before calling `setSelectedTicketId(...)`, call `clearMultiSelection()` from the store. This satisfies the "arrow keys clear multi-selection" requirement without changing navigation logic.

**Order of implementation**
1. Store changes (foundation for everything)
2. Backend batch endpoints (needed by frontend batch panel)
3. `Swimlane` header checkbox + `TicketCard` shift-click
4. `BatchDetailPanel` in `TicketDetail`
5. `WorkScreen` arrow-key clear

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T21:27Z | — | new | apm |
| 2026-04-02T22:46Z | new | groomed | apm |
| 2026-04-02T22:47Z | groomed | in_design | philippepascal |
| 2026-04-02T22:51Z | in_design | specd | claude-0402-2300-s9k2 |
| 2026-04-02T22:55Z | specd | ready | apm |
| 2026-04-02T22:59Z | ready | in_progress | philippepascal |
| 2026-04-02T23:02Z | in_progress | implemented | claude-0402-2359-w7k4 |
| 2026-04-02T23:10Z | implemented | ammend | apm |
