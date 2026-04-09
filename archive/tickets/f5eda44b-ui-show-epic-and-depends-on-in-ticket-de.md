+++
id = "f5eda44b"
title = "UI: show epic and depends_on in ticket detail panel"
state = "closed"
priority = 2
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "58435"
branch = "ticket/f5eda44b-ui-show-epic-and-depends-on-in-ticket-de"
created_at = "2026-04-01T21:56:10.584818Z"
updated_at = "2026-04-02T19:08:18.601481Z"
+++

## Spec

### Problem

The ticket detail panel (`apm-ui/src/components/TicketDetail.tsx`) renders core fields — title, state, effort, risk, priority — but has no awareness of `epic` or `depends_on`. Engineers cannot tell from the UI which epic a ticket belongs to, or which tickets it is waiting on before it can be dispatched.

The underlying `Frontmatter` struct in `apm-core/src/ticket.rs` does not yet declare `epic` or `depends_on` fields, so they are stripped during parsing even when present in the TOML frontmatter. Adding them to the struct is the minimal server-side change needed; the `TicketDetailResponse` already flattens `Frontmatter`, so the new fields will appear in the API automatically.

On the UI side, two small features are needed in the detail panel header: a clickable epic label (that sets the epic filter on the supervisor board so engineers can quickly scope to the same epic) and a dependency list (ticket IDs that link to each dep's detail panel, with strikethrough on resolved deps).

### Acceptance criteria

- [x] When a ticket has no `epic` field, the detail panel shows no epic row
- [x] When a ticket has an `epic` field, the detail panel shows a labelled row with the epic ID value
- [x] Clicking the epic label sets `epicFilter` in the layout store to that epic ID
- [x] When `epicFilter` is set in the layout store, the supervisor board hides tickets whose `epic` field does not match (tickets with no `epic` field are also hidden)
- [x] Clicking the epic label a second time on the same ticket while the filter is already active clears the filter (toggle behaviour)
- [x] When a ticket has no `depends_on` field (or an empty array), the detail panel shows no dependencies row
- [x] When a ticket has a `depends_on` field, the detail panel lists each dep ticket ID
- [x] Clicking a dep ticket ID in the detail panel sets `selectedTicketId` in the layout store to that dep's full ID, opening its detail panel
- [x] Dep tickets that are absent from `blocking_deps` in the API response are shown with strikethrough text
- [x] Dep tickets that appear in `blocking_deps` in the API response are shown without strikethrough
- [x] A dep ticket ID that does not resolve to any known ticket renders as plain text (no link, no crash)
- [x] Existing tickets without `epic` or `depends_on` in their frontmatter continue to load and display correctly

### Out of scope

- `target_branch` frontmatter field (used by `apm start` for epic branching, not a UI concern here)
- Epic creation, listing, or management commands (`apm epic new`, `apm epic list`, `apm epic show`, `apm epic close`)
- New-ticket modal epic/depends_on input fields (separate ticket)
- Lock icon on ticket cards in the queue or supervisor board for unresolved deps (separate ticket)
- Epic column in the priority queue panel (separate ticket)
- Engine scheduling changes that block dispatch on `depends_on` (separate ticket)
- A standalone epic filter dropdown control on the supervisor board — the filter is set exclusively by clicking the epic label in the detail panel
- `GET /api/epics` server routes — no epic-specific API routes are needed for this feature

### Approach

Five files change. Apply in this order:

#### 1. `apm-core/src/ticket.rs` — extend `Frontmatter`

Add two optional fields to the `Frontmatter` struct:

```rust
pub epic: Option<String>,
pub depends_on: Option<Vec<String>>,
```

Decorate both with `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing ticket files without these fields deserialize and round-trip correctly. No migration needed.

`TicketDetailResponse` and `TicketResponse` in `apm-server/src/main.rs` both use `#[serde(flatten)] frontmatter`, so the new fields appear in all existing API responses automatically — no server-side struct changes required beyond those in ticket da95246d (which adds `blocking_deps`).

#### 2. `apm-ui/src/store/useLayoutStore.ts` — add epic filter state

Add to the `LayoutStore` interface and implementation:

```ts
epicFilter: string | null
setEpicFilter: (id: string | null) => void
```

Initialise `epicFilter: null`. Setter: `set({ epicFilter: id })`.

#### 3. `apm-ui/src/components/supervisor/types.ts` — extend `Ticket` type

Add the following to the `Ticket` type to match the extended API response:

```ts
epic?: string
depends_on?: string[]
blocking_deps?: Array<{ id: string; state: string }>
```

#### 4. `apm-ui/src/components/supervisor/SupervisorView.tsx` — apply epic filter

Read from store: `const epicFilter = useLayoutStore((s) => s.epicFilter)`.

In the `columns` `useMemo`, after the existing `agentFilter` block:

```ts
if (epicFilter !== null) {
  filtered = filtered.filter((t) => t.epic === epicFilter)
}
```

Add `epicFilter` to the `useMemo` dependency array.

#### 5. `apm-ui/src/components/TicketDetail.tsx` — render epic and depends_on rows

Update the `TicketDetail` interface: add `epic?: string`, `depends_on?: string[]`, and `blocking_deps?: Array<{ id: string; state: string }>`.

Add to component reads: `epicFilter` and `setEpicFilter` from layout store.

**Epic row** — render below the state/E-R-P badge row, only when `data.epic` is present:
- Label "Epic", value is a `<button>` showing the epic ID
- Click toggles: if `epicFilter === data.epic` call `setEpicFilter(null)`, else call `setEpicFilter(data.epic)`
- When filter is active for this epic, apply a blue-border highlight to the button

**Depends on row** — render below the epic row, only when `data.depends_on?.length` is truthy:
- Label "Depends on"
- Build a set of blocking dep IDs: `const blockingSet = new Set((data.blocking_deps ?? []).map(d => d.id))`
- For each dep ID, look it up in the React Query cache via `useQueryClient().getQueryData<Ticket[]>(['tickets'])` to determine if it is a known ticket
- If found: render a `<button>` that calls `setSelectedTicketId(fullId)`. Apply `line-through` class if the dep ID is **absent** from `blockingSet` (i.e. it is resolved — not blocking)
- If not found: render the raw ID as plain text (no crash)
- Use the existing `useQueryClient` import; do not add a new `useQuery` call

Resolution is determined entirely by `blocking_deps` — do not check state field names.

#### Tests

Add a unit test in `apm-core/src/ticket.rs` or `apm-core/tests/`: parse a ticket with `epic = "ab12cd34"` and `depends_on = ["cd56ef78"]` in frontmatter and assert both fields deserialize correctly. Parse a ticket without these fields and assert both are `None`. All existing tests must continue to pass.

### Open questions


### Amendment requests

- [x] Delete the duplicate "### 3." and "### 4." sections at the bottom of the spec. The "### 3." still instructs the worker to apply `line-through` when state is `implemented`, `accepted`, or `closed` — hardcoded state names. The corrected Approach above (using `blocking_deps` for resolution) is authoritative; the stale duplicate sections must be removed.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:53Z | groomed | in_design | philippepascal |
| 2026-04-02T00:56Z | in_design | specd | claude-0402-0100-b7e2 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:42Z | ammend | in_design | philippepascal |
| 2026-04-02T01:45Z | in_design | specd | claude-0402-0200-c9f1 |
| 2026-04-02T01:56Z | specd | ammend | philippepascal |
| 2026-04-02T01:56Z | ammend | in_design | philippepascal |
| 2026-04-02T01:59Z | in_design | specd | claude-0402-0200-spec1 |
| 2026-04-02T02:03Z | specd | ammend | apm |
| 2026-04-02T02:11Z | ammend | in_design | philippepascal |
| 2026-04-02T02:14Z | in_design | specd | claude-0402-0215-d4e1 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:36Z | ready | in_progress | philippepascal |
| 2026-04-02T06:44Z | in_progress | implemented | claude-0402-0640-w9k2 |
| 2026-04-02T19:08Z | implemented | closed | apm-sync |