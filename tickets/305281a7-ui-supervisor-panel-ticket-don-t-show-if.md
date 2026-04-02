+++
id = "305281a7"
title = "UI supervisor panel ticket don't show if they are part of epic"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "apm"
agent = "51125"
branch = "ticket/305281a7-ui-supervisor-panel-ticket-don-t-show-if"
created_at = "2026-04-02T22:32:22.237758Z"
updated_at = "2026-04-02T22:50:48.892807Z"
+++

## Spec

### Problem

The supervisor board currently shows every ticket regardless of whether it belongs to an epic. This creates visual noise: a ticket already tracked under an epic appears twice — once inside the epic and once in the top-level board. As the number of epics and their child tickets grows, the board becomes cluttered and harder to scan.

Tickets that belong to an epic are managed at the epic level. For the supervisor's top-level view the right unit of work is the epic, not each constituent ticket. Epic-member tickets should therefore be hidden from the default board view, with an opt-in toggle to reveal them when needed (e.g. to inspect all blocked tickets across every epic).

### Acceptance criteria

- [ ] By default (on first load), tickets whose `epic` field is non-null are not shown in the supervisor board swimlanes
- [ ] A "Show epic tickets" checkbox appears in the supervisor filter bar alongside the existing "Show closed" checkbox
- [ ] Checking "Show epic tickets" reveals epic-member tickets in the board
- [ ] Unchecking "Show epic tickets" hides epic-member tickets again
- [ ] When the epic filter dropdown is set to a specific epic, only that epic's tickets are shown regardless of the "Show epic tickets" toggle state
- [ ] When an epic filter is active and cleared, the board returns to hiding epic-member tickets (if the toggle is still unchecked)

### Out of scope

- Showing epic summary cards or rows in the board (epic-level overview widgets)
- Persisting the "Show epic tickets" toggle to localStorage across page reloads
- Server-side filtering — this is a purely client-side change
- Changes to the epic detail view or the epic filter dropdown itself
- Filtering logic for any panel other than the supervisor board

### Approach

Two files change; no server changes required.

**1. `apm-ui/src/store/useLayoutStore.ts`**
- Add `showEpicTickets: boolean` field, defaulting to `false`
- Add `setShowEpicTickets: (v: boolean) => void` setter

**2. `apm-ui/src/components/supervisor/SupervisorView.tsx`**

*Read the store value:*
```ts
const showEpicTickets = useLayoutStore((s) => s.showEpicTickets)
const setShowEpicTickets = useLayoutStore((s) => s.setShowEpicTickets)
```

*Extend the `columns` useMemo filter block* (after the existing epicFilter block):
```ts
// hide epic-member tickets unless the toggle is on or we are already
// scoped to a specific epic
if (!showEpicTickets && epicFilter === null) {
  filtered = filtered.filter((t) => t.epic == null)
}
```
Add `showEpicTickets` to the dependency array.

*Add the checkbox to the filter bar* (next to the existing "Show closed" label):
```tsx
<label className="flex items-center gap-1.5 text-xs cursor-pointer select-none">
  <input
    type="checkbox"
    checked={showEpicTickets}
    onChange={(e) => setShowEpicTickets(e.target.checked)}
    className="rounded"
  />
  Show epic tickets
</label>
```

No tests are needed beyond manual verification — this is a purely presentational filter with no business logic to unit-test. The existing integration test suite does not cover UI components.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T22:32Z | — | new | apm |
| 2026-04-02T22:32Z | new | groomed | apm |
| 2026-04-02T22:48Z | groomed | in_design | philippepascal |