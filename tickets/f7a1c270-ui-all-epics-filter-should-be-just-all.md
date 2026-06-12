+++
id = "f7a1c270"
title = "UI: All Epics filter should be just All"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7a1c270-ui-all-epics-filter-should-be-just-all"
created_at = "2026-06-10T02:50:51.566252Z"
updated_at = "2026-06-12T23:00:27.737167Z"
+++

## Spec

### Problem

The epic filter dropdown in the board's toolbar shows "All epics" as its default (unfiltered) option. This label is inconsistent with the intent: selecting it shows every ticket regardless of epic, which corresponds to `apm list --all` semantics — all tickets including those in terminal (closed) states. However, the current implementation only shows non-closed tickets when the option is selected, requiring a separate "Show closed" checkbox to surface closed tickets. The board therefore diverges from `apm list --all` when the epic filter is in its default state.

The fix has two parts: rename the option label to "All" (dropping the redundant "epics" qualifier), and ensure that selecting "All" includes closed tickets automatically, matching the full set `apm list --all` returns.

### Acceptance criteria

- [x] The epic filter dropdown default option reads "All" (not "All epics")
- [x] When "All" is selected, the board fetches and displays closed/terminal tickets without requiring the "Show closed" checkbox to be checked
- [x] When a specific epic is selected, closed tickets are hidden unless "Show closed" is also checked
- [x] When "No epic" is selected, closed tickets are hidden unless "Show closed" is also checked
- [x] The "Show closed" checkbox remains visible and continues to function as a way to include closed tickets when a specific epic or "No epic" is active

### Out of scope

- Renaming other filter labels ("All states", "All owners", "All authors")
- Changing server-side API endpoint behaviour
- Any change to the "No epic" filter semantics beyond what is already specified

### Approach

All changes are in `apm-ui/src/components/supervisor/SupervisorView.tsx` and `apm-ui/src/components/PriorityQueuePanel.tsx`.

1. **Derive `includeClosed`** — replace direct use of `showClosed` in the ticket query and visible-states memo with a derived boolean:

   ```ts
   const includeClosed = showClosed || epicFilter === null
   ```

   `epicFilter === null` is the "All" state (no epic selected). `showClosed` retains its role for other filter selections.

2. **Update the ticket query** — change `queryKey` and `queryFn` to use `includeClosed` instead of `showClosed`:

   ```ts
   queryKey: ['tickets', includeClosed],
   queryFn: () => fetchTickets(includeClosed),
   ```

3. **Update `visibleStates`** — replace `showClosed` with `includeClosed` so the closed swimlane appears automatically in "All" mode:

   ```ts
   if (includeClosed) base.push('closed')
   ```

   Also add `epicFilter` to the dependency array alongside `includeClosed`.

4. **Rename the option label in SupervisorView.tsx** — change line 239:

   ```tsx
   <option value="">All epics</option>
   ```
   to:
   ```tsx
   <option value="">All</option>
   ```

5. **Rename the option label in PriorityQueuePanel.tsx** — change line 279:

   ```tsx
   <option value="">All epics</option>
   ```
   to:
   ```tsx
   <option value="">All</option>
   ```

6. **`hasActiveFilters`** — no change needed; `epicFilter !== null` already stays false when "All" is selected, and the closed state becoming visible is expected rather than an "active filter".

No backend changes, no store changes, no new files.

### Open questions


### Amendment requests

- [x] A second, identical epic-filter dropdown exists at apm-ui/src/components/supervisor/PriorityQueuePanel.tsx:279 (<option value="">All epics</option>), which the current Approach neither renames nor scopes out. AC #1 ('the epic filter dropdown reads All') is phrased generally, so this label would remain and AC #1 could be judged unmet. Resolve it one of two ways: (a) extend the Approach to also rename PriorityQueuePanel.tsx:279 from 'All epics' to 'All' (same one-line label change, same intent), OR (b) explicitly add PriorityQueuePanel's dropdown to Out of scope and narrow AC #1 to the supervisor board's dropdown only. Pick (a) unless there's a reason the priority-queue panel should keep its own label.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-10T02:50Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:13Z | groomed | in_design | philippepascal |
| 2026-06-12T08:17Z | in_design | specd | claude |
| 2026-06-12T22:32Z | specd | ammend | philippepascal |
| 2026-06-12T22:33Z | ammend | in_design | philippepascal |
| 2026-06-12T22:33Z | in_design | specd | claude |
| 2026-06-12T22:53Z | specd | ready | philippepascal |
| 2026-06-12T23:00Z | ready | in_progress | philippepascal |