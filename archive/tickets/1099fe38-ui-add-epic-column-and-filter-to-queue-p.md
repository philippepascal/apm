+++
id = "1099fe38"
title = "UI: add epic column and filter to queue panel"
state = "closed"
priority = 2
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "39624"
branch = "ticket/1099fe38-ui-add-epic-column-and-filter-to-queue-p"
created_at = "2026-04-01T21:56:20.710748Z"
updated_at = "2026-04-02T19:06:23.656182Z"
+++

## Spec

### Problem

The queue panel table in `PriorityQueuePanel.tsx` shows tickets without any indication of which epic they belong to, and provides no way to filter by epic. When multiple epics are in flight, tickets from all epics are interleaved and there is no way to focus on a single epic's work queue.

The fix is two additive changes to the queue panel: (1) an **Epic** column that shows the short 8-char epic ID for tickets inside an epic, or "—" for free tickets; and (2) an epic filter dropdown that hides tickets not belonging to the selected epic.

The `epic` field does not yet exist on `Frontmatter` (apm-core) or `QueueEntry` (apm-server), so both must be extended. The UI change is purely additive — no existing columns or interactions change.

### Acceptance criteria

- [x] The queue table has an "Epic" column header between "State" and "E" (effort)
- [x] Each queue row shows the short 8-char epic ID when `epic` is set in the ticket frontmatter
- [x] Each queue row shows "—" in the Epic column when the ticket has no `epic` field
- [x] A filter dropdown above the queue table lists all distinct epic IDs present in the current queue, plus an "All epics" option
- [x] Selecting an epic ID from the dropdown hides all rows whose Epic column does not match that value
- [x] Selecting "All epics" restores the unfiltered view
- [x] The epic filter has no effect on drag-to-reorder or arrow-key reordering (reordering still works on the unfiltered, full queue)
- [x] Tickets with no `epic` field in frontmatter parse and display correctly — the absence of the field does not cause a parse error or panic

### Out of scope

- Supervisor board epic filter (separate ticket per docs/epics.md)
- Engine controls epic selector (separate ticket per docs/epics.md)
- Ticket card lock icon for unresolved `depends_on` (separate ticket)
- New ticket modal epic dropdown (separate ticket)
- Ticket detail panel epic/depends_on display (separate ticket)
- `GET /api/epics` route (not needed here; epic list is derived from queue data)
- `depends_on` and `target_branch` frontmatter fields (not needed for this ticket)
- Persisting the epic filter selection across page reloads

### Approach

**1. apm-core/src/ticket.rs** - Add optional epic field to Frontmatter after focus_section. Use serde skip_serializing_if Option::is_none. Tickets without epic in frontmatter deserialize fine because Option defaults to None. Update fake_ticket test helper and any other Frontmatter constructors in test code to include epic: None.

**2. apm-server/src/queue.rs** - Add epic: Option<String> to QueueEntry struct. Populate it from fm.epic.clone() in the mapping closure.

**3. apm-ui/src/components/PriorityQueuePanel.tsx** - Four sub-changes:

(a) Extend QueueEntry interface: add epic field as optional string.

(b) Add epicFilter state (string or null, default null). Derive availableEpics array with useMemo by collecting distinct non-null entry.epic values from displayQueue, sorted alphabetically.

(c) Compute filteredQueue: when epicFilter is set, filter displayQueue to rows where entry.epic matches; otherwise use displayQueue unchanged. Use filteredQueue only for SortableRow rendering. Keep displayQueue (unfiltered) for SortableContext items list and all reorder operations, so drag and keyboard reordering continue to act on the full queue.

(d) When availableEpics is non-empty, render an epic filter bar above the DndContext block. A select element with All epics option (empty value) plus one option per epic ID. Selecting empty sets filter to null; selecting an ID sets filter to that string.

(e) In the table header, insert an Epic th between State and E columns.

(f) In SortableRow, add epic to props and insert a td after the state cell showing entry.epic or the dash character in font-mono text-gray-500 text-[10px] style.

**Order of changes:**
1. apm-core/src/ticket.rs - add epic field
2. apm-server/src/queue.rs - add epic field, fix test helpers
3. apm-ui/src/components/PriorityQueuePanel.tsx - UI
4. cargo test --workspace must pass

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:56Z | groomed | in_design | philippepascal |
| 2026-04-02T00:59Z | in_design | specd | claude-0402-0057-spec1 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:45Z | ready | in_progress | philippepascal |
| 2026-04-02T06:47Z | in_progress | implemented | claude-0402-0645-w1k9 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |