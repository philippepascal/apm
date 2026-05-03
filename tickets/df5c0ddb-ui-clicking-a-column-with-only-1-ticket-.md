+++
id = "df5c0ddb"
title = "UI: clicking a column with only 1 ticket doesn't select ticket"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/df5c0ddb-ui-clicking-a-column-with-only-1-ticket-"
created_at = "2026-05-03T19:01:41.363530Z"
updated_at = "2026-05-03T19:27:14.528623Z"
+++

## Spec

### Problem

When a user clicks the checkbox in a Swimlane column header, the handler calls `selectColumn(columnIds)`, which appends the ticket IDs to `selectedTicketIds` in the Zustand store. This triggers the multi-select visual state (blue-400 ring on the ticket card) but does **not** set `selectedTicketId` — the field that opens the ticket in the detail panel.

For columns with two or more tickets this is the correct outcome: all tickets get the multi-select ring, none opens individually in the detail panel. But for a column with exactly one ticket, the expected behaviour is equivalent to clicking that ticket card directly — the ticket should be fully selected (`selectedTicketId` set, detail panel opened, blue-500 ring).

Currently the column-header checkbox click for a 1-ticket column only adds the ID to `selectedTicketIds` and leaves `selectedTicketId` unchanged (null or pointing at a different ticket). The ticket appears with a multi-select ring but does not open in the detail panel, which users experience as "not selected."

A secondary issue: `allSelected` is computed solely from `selectedTicketIds`, so when the ticket is already single-selected (via `setSelectedTicketId`, which clears `selectedTicketIds`), the checkbox renders as unchecked and the deselect path (`deselectColumn`) is a no-op, leaving the selection state inconsistent.

### Acceptance criteria

- [ ] Clicking the column-header checkbox of a column containing exactly 1 ticket sets `selectedTicketId` to that ticket's ID in the layout store
- [ ] After clicking the column-header checkbox of a 1-ticket column, the detail panel displays that ticket
- [ ] After clicking the column-header checkbox of a 1-ticket column, the ticket card displays the single-select ring (ring-blue-500), not the multi-select ring (ring-blue-400)
- [ ] The column-header checkbox of a 1-ticket column renders as checked when that ticket is the current `selectedTicketId`
- [ ] Clicking the column-header checkbox of a 1-ticket column a second time (when the ticket is already selected) clears `selectedTicketId` (sets it to null) and the detail panel closes
- [ ] Clicking the column-header checkbox of a column containing 2 or more tickets continues to multi-select all tickets in that column (populates `selectedTicketIds`), unchanged from current behaviour

### Out of scope

- Behaviour of the column-header checkbox for columns with 2+ tickets (already works correctly)\n- Keyboard navigation selection (arrow keys in WorkScreen), which uses setSelectedTicketId directly and is unaffected\n- Shift-click range selection within a column (selectTicketRange), unaffected\n- Any changes to useLayoutStore.ts store actions

### Approach

Single file changes: `apm-ui/src/components/supervisor/Swimlane.tsx`

#### 1. Extend the store destructure

Add `selectedTicketId` and `setSelectedTicketId` to the values pulled from `useLayoutStore`:

```typescript
const { selectedTicketId, selectedTicketIds, selectColumn, deselectColumn, setSelectedTicketId } = useLayoutStore()
```

#### 2. Update `allSelected` to cover the single-ticket single-select case

The current expression only checks `selectedTicketIds`. When a ticket was opened via a card click, `setSelectedTicketId` clears `selectedTicketIds` to `[]`, so `allSelected` evaluates to `false` even though the ticket is visually selected.

Replace:
```typescript
const allSelected = tickets.length > 0 && columnIds.every((id) => selectedTicketIds.includes(id))
```
With:
```typescript
const allSelected =
  tickets.length > 0 &&
  (columnIds.every((id) => selectedTicketIds.includes(id)) ||
    (columnIds.length === 1 && selectedTicketId === columnIds[0]))
```

#### 3. Branch `handleHeaderCheckbox` on column size

For a 1-ticket column use `setSelectedTicketId` (single-select semantics).  
For a multi-ticket column keep the existing `selectColumn`/`deselectColumn` (multi-select semantics).

Replace:
```typescript
function handleHeaderCheckbox() {
  if (allSelected) {
    deselectColumn(columnIds)
  } else {
    selectColumn(columnIds)
  }
}
```
With:
```typescript
function handleHeaderCheckbox() {
  if (columnIds.length === 1) {
    // Single-ticket column: toggle single-select (opens/closes detail panel)
    if (allSelected) {
      setSelectedTicketId(null)
    } else {
      setSelectedTicketId(columnIds[0])
    }
  } else {
    if (allSelected) {
      deselectColumn(columnIds)
    } else {
      selectColumn(columnIds)
    }
  }
}
```

No changes required in `useLayoutStore.ts` or any other file. `someSelected` and the `indeterminate` effect are unaffected: for a 1-ticket column `allSelected` and `someSelected` always agree (both true or both false), so `indeterminate` stays `false`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-03T19:01Z | — | new | philippepascal |
| 2026-05-03T19:01Z | new | groomed | philippepascal |
| 2026-05-03T19:04Z | groomed | in_design | philippepascal |
| 2026-05-03T19:13Z | in_design | specd | claude-0503-1904-9e70 |
| 2026-05-03T19:27Z | specd | ready | philippepascal |
