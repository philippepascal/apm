+++
id = "2ef15663"
title = "UI: filter the epic bar indicating epics needing refresh"
state = "in_design"
priority = 0
effort = 1
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2ef15663-ui-filter-the-epic-bar-indicating-epics-"
created_at = "2026-06-01T17:18:45.877184Z"
updated_at = "2026-06-01T17:20:15.220583Z"
+++

## Spec

### Problem

The Supervisor board has an epic bar — a row of amber/red badges, one per stale epic (branches that are behind `HEAD` and need a `git pull`). When a user applies an epic filter using the dropdown, the ticket swimlanes narrow to only the selected epic's tickets, but the epic bar continues to show all stale epics regardless of the filter. The bar and the board are out of sync.

The desired behaviour: the epic bar should reflect the same epic scope as the rest of the board. When a specific epic filter is active, show only that epic's badge (if it is stale). When the "No epic" filter is active, hide the bar entirely — tickets with no epic cannot belong to a stale epic branch. When no filter is active, the bar behaves exactly as it does today.

### Acceptance criteria

- [ ] When no epic filter is active, the epic bar shows all stale epics that appear in any ticket (existing behaviour unchanged)
- [ ] When a specific epic ID is selected in the epic filter and that epic is stale, the epic bar shows exactly one badge for that epic
- [ ] When a specific epic ID is selected in the epic filter and that epic is not stale, the epic bar is hidden
- [ ] When the "No epic" filter (`__none__`) is active, the epic bar is hidden
- [ ] Stale epics that belong to a different epic than the active filter are not shown in the epic bar while that filter is active

### Out of scope

- Filtering the epic bar by owner, author, or state filters — only the epic filter is in scope
- Changes to how `epicIdsInTickets` is computed (it continues to derive from all tickets, not just visible ones)
- Any changes to the epic dropdown options or their ordering
- Backend / API changes

### Approach

**File:** `apm-ui/src/components/supervisor/SupervisorView.tsx`

The only change is in the inline IIFE that renders the epic bar (currently around line 255). Replace the single-condition `staleEpics` filter with one that also gates on `epicFilter`:

```tsx
// Before
const staleEpics = epics.filter((ep) => ep.behind_count > 0 && epicIdsInTickets.has(ep.id))

// After
const staleEpics = epics.filter((ep) => {
  if (ep.behind_count === 0) return false
  if (!epicIdsInTickets.has(ep.id)) return false
  if (epicFilter === '__none__') return false
  if (epicFilter !== null && epicFilter !== ep.id) return false
  return true
})
```

`epicFilter` is already in the closure (declared at line 45 via `useLayoutStore`), so no new state, props, or imports are needed. The existing `staleEpics.length === 0` guard already hides the bar when the filtered list is empty, covering the cases where the selected epic is not stale and where `__none__` is active.

No tests exist for this component's render output today; no test changes are required. Manual verification: apply each epic filter permutation and confirm the bar matches the expected output from the acceptance criteria.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-01T17:18Z | — | new | philippepascal |
| 2026-06-01T17:18Z | new | groomed | philippepascal |
| 2026-06-01T17:18Z | groomed | in_design | philippepascal |