+++
id = "711e717e"
title = "UI: add epic filter to supervisor board filter bar"
state = "closed"
priority = 2
effort = 2
risk = 2
author = "claude-0401-2145-a8f3"
agent = "80465"
branch = "ticket/711e717e-ui-add-epic-filter-to-supervisor-board-f"
created_at = "2026-04-01T21:56:24.806901Z"
updated_at = "2026-04-02T19:07:09.986371Z"
+++

## Spec

### Problem

The supervisor board filter bar has state and agent filters but no epic filter. When multiple epics are active, all their tickets appear together, making it impossible for the supervisor to isolate a single epic's work in the board view.

The desired behaviour: an epic dropdown in the filter bar (beside the existing state and agent dropdowns) lets the supervisor select one epic and hide all tickets that belong to other epics, or select All to restore the default view. The dropdown is populated from GET /api/epics.

This is a UI-only ticket. Two server-side prerequisites must land first: ticket d877bd37 adds the epic field to Frontmatter (which makes GET /api/tickets return the epic field on each ticket), and ticket 54b043f7 implements GET /api/epics. This ticket implements only the React changes in apm-ui.

### Acceptance criteria

- [x] An "Epic" dropdown appears in the supervisor board filter bar, positioned after the "All agents" dropdown
- [x] The dropdown contains an "All epics" option that is selected by default and shows all tickets
- [x] The dropdown options are populated from `GET /api/epics` and show each epic's title
- [x] Selecting an epic hides all ticket cards whose `epic` field does not match the selected epic id
- [x] Tickets with no `epic` field are hidden when any specific epic is selected
- [x] Selecting "All epics" after a specific epic restores the full board view
- [x] The "No tickets match the current filters" empty state appears when an epic is selected and no tickets match
- [x] The epic filter composes with the existing state, agent, and search filters (all active simultaneously)
- [x] `GET /api/epics` returns a JSON array; each element has `id`, `title`, and `branch` string fields
- [x] `GET /api/epics` returns an empty array when no `epic/*` branches exist
- [x] The dropdown renders but shows only "All epics" when `GET /api/epics` returns an empty array

### Out of scope

- GET /api/epics server route — covered by ticket 54b043f7
- Adding epic field to Frontmatter struct — covered by ticket d877bd37
- Epic filter in the Queue panel — covered by ticket 1099fe38
- Epic column in the Queue panel
- Epic selector in Engine controls — covered by ticket ea172f4a
- POST /api/epics or GET /api/epics/:id server routes
- Clickable epic label in Ticket detail panel — covered by ticket f5eda44b
- Ticket lock icon for unresolved depends_on entries — covered by ticket da95246d
- Any changes to the work engine epic scheduling

### Approach

This ticket is UI-only. The server-side prerequisites are covered by other tickets:
- `d877bd37` adds `epic` (and `target_branch`, `depends_on`) to `Frontmatter`; once merged, `GET /api/tickets` responses carry the `epic` field automatically.
- `54b043f7` adds `GET /api/epics` to apm-server.

This ticket must be implemented after both of those are merged, or developed in parallel with stubs.

Two files change.

**1. `apm-ui/src/components/supervisor/types.ts`**

Add one optional field to `Ticket`:

```typescript
epic?: string
```

**2. `apm-ui/src/components/supervisor/SupervisorView.tsx`**

a) Add an `Epic` type and `fetchEpics` function after `fetchTickets` (line 27):

```typescript
interface Epic { id: string; title: string; branch: string }

async function fetchEpics(): Promise<Epic[]> {
  const res = await fetch('/api/epics')
  if (!res.ok) return []
  return res.json()
}
```

b) Inside `SupervisorView`, add state and query after the `agentFilter` line (line 41):

```typescript
const [epicFilter, setEpicFilter] = useState<string | null>(null)
const { data: epics = [] } = useQuery({ queryKey: ['epics'], queryFn: fetchEpics })
```

c) In the `columns` useMemo, after the `agentFilter` block (lines 91–93), add:

```typescript
if (epicFilter !== null) {
  filtered = filtered.filter((t) => t.epic === epicFilter)
}
```

Add `epicFilter` to the dependency array (line 104).

d) Update `hasActiveFilters` (line 106) to include `|| epicFilter !== null`.

e) Add the dropdown to the filter bar JSX after the agent `<select>` (after line 176):

```tsx
<select
  value={epicFilter ?? ''}
  onChange={(e) => setEpicFilter(e.target.value || null)}
  className="h-7 px-1.5 text-xs border rounded bg-white focus:outline-none focus:ring-1 focus:ring-blue-400"
>
  <option value="">All epics</option>
  {epics.map((ep) => (
    <option key={ep.id} value={ep.id}>{ep.title || ep.id}</option>
  ))}
</select>
```

**Tests**: `cargo test --workspace` must pass. This is a pure React state/render change; no new
Rust or JS tests are required.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:57Z | groomed | in_design | philippepascal |
| 2026-04-02T01:06Z | in_design | specd | claude-0401-2300-spec1 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:34Z | ready | in_progress | philippepascal |
| 2026-04-02T06:36Z | in_progress | implemented | claude-0401-0634-w711 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |