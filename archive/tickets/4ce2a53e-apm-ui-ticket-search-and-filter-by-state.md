+++
id = "4ce2a53e"
title = "apm-ui: ticket search and filter (by state, agent, text)"
state = "closed"
priority = 25
effort = 3
risk = 1
author = "apm"
agent = "53554"
branch = "ticket/4ce2a53e-apm-ui-ticket-search-and-filter-by-state"
created_at = "2026-03-31T06:13:17.849783Z"
updated_at = "2026-04-01T07:12:35.102929Z"
+++

## Spec

### Problem

The supervisor swimlane view (Step 5) only shows tickets grouped by supervisor-actionable states, with no way to search by text, filter to a specific state or agent, or surface closed/cancelled tickets. Supervisors who need to find a specific ticket by content, check on a specific agent's work, or review completed tickets must fall back to the CLI.

The fix is a filter bar above the swimlane grid that provides parity with the two most-used CLI flags: `apm list --state <state>` (narrow to one state, including any non-supervisor state) and `apm list --all` (expose closed/cancelled). Text search and an agent picker round out the filtering surface.

The `GET /api/tickets` response already includes a `body` field (Step 2 spec), so all filtering can run client-side against TanStack Query's cached data — no new backend endpoints are needed.

### Acceptance criteria

- [x] A filter bar is visible at the top of the SupervisorView (middle column) containing a text search input, a state dropdown, an agent dropdown, and a show-closed toggle
- [x] Typing in the text search input filters ticket cards to those whose title or body contains the query string (case-insensitive)
- [x] Clearing the text input restores all ticket cards that matched before the search was applied
- [x] Selecting a state in the state dropdown shows only the swimlane for that state (any valid workflow state, not limited to supervisor-actionable ones)
- [x] Clearing the state dropdown restores the default supervisor-actionable-states-only swimlane view
- [x] Selecting an agent in the agent dropdown shows only tickets where agent equals the selected value, across all visible swimlanes
- [x] Clearing the agent dropdown restores tickets for all agents
- [x] Enabling the show-closed toggle reveals the closed swimlane in addition to the default supervisor-actionable states
- [x] Disabling the show-closed toggle hides the closed swimlane
- [x] When no tickets match the combined active filters, an empty-state message replaces the swimlane grid
- [x] The agent dropdown options are derived from the unique non-empty agent values present in the loaded ticket list
- [x] Multiple filters active simultaneously are combined with AND logic (all conditions must be satisfied)
- [x] Filter state is held in local React component state and resets to defaults on page reload

### Out of scope

- Persisting filter state to localStorage, URL params, or any other storage across page reloads
- Server-side filtering or new API endpoints (all filtering is client-side on cached data)
- Visual badges for open questions or amendment requests on ticket cards (covered by ticket ebae68e2, Step 14c)
- Log tail viewer (covered by ticket e9ba2503, Step 14b)
- Keyboard navigation across swimlanes (covered by Step 6)
- Debouncing or throttling the text search input (unnecessary at ticket scale)

### Approach

**Modify `apm-ui/src/components/supervisor/SupervisorView.tsx`**

Add four pieces of local state at the top of the component:
- `searchText: string` (default `""`)
- `stateFilter: string | null` (default `null`)
- `agentFilter: string | null` (default `null`)
- `showClosed: boolean` (default `false`)

**Derive `visibleStates`** — the ordered list of swimlane state columns to consider:
1. Start with `SUPERVISOR_STATES` (the existing hard-coded list: question, specd, blocked, implemented, accepted)
2. If `showClosed` is true, append `['closed']`
3. If `stateFilter` is non-null, override to just `[stateFilter]` (stateFilter takes full precedence)

**Derive `availableAgents: string[]`** — unique, non-empty, sorted agent values from all loaded tickets. Used to populate the agent dropdown.

**Filter tickets within each swimlane** — before passing tickets to `Swimlane`, apply:
- Agent filter: if `agentFilter` is set, keep only `ticket.agent === agentFilter`
- Text filter: if `searchText.trim()` is non-empty, keep only tickets where `ticket.title` or `ticket.body ?? ""` includes `searchText` (case-insensitive via `.toLowerCase()`)

A swimlane is still hidden when zero tickets pass the filter (existing behaviour unchanged).

**Add a `FilterBar` section** at the top of the SupervisorView render, above the swimlane row. Either inline in `SupervisorView.tsx` or extracted to `components/supervisor/FilterBar.tsx` (prefer inline if it stays readable).

FilterBar contains (using shadcn components):
- `Input` for text search; show a clear (×) button when `searchText` is non-empty
- `Select` for state: a fixed option list of all workflow states (`new`, `in_design`, `question`, `specd`, `ready`, `in_progress`, `blocked`, `implemented`, `accepted`, `closed`) plus a "All states" default option
- `Select` for agent: options from `availableAgents` plus an "All agents" default option
- `Switch` (or `Checkbox`) labelled "Show closed"

**Empty-state message**: when `visibleStates` produces zero non-empty swimlanes after filtering, render a short message such as "No tickets match the current filters" in place of the swimlane grid.

**No backend changes required.** The `GET /api/tickets` response already includes a `body` field (per Step 2 spec), so all filtering is purely client-side.

### Open questions



### Amendment requests

- [x] Remove `'cancelled'` from the show-closed toggle's extra-states list in the Approach — there is no `cancelled` state in the workflow config. The list should be `['closed']` only.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:18Z | new | in_design | philippepascal |
| 2026-03-31T07:23Z | in_design | specd | claude-0331-0800-b7f2 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T18:23Z | ammend | in_design | philippepascal |
| 2026-03-31T18:27Z | in_design | specd | claude-0331-1430-s9w2 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T06:38Z | ready | in_progress | philippepascal |
| 2026-04-01T06:42Z | in_progress | implemented | claude-0401-0639-8f48 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |