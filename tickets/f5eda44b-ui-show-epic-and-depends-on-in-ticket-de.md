+++
id = "f5eda44b"
title = "UI: show epic and depends_on in ticket detail panel"
state = "in_design"
priority = 2
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/f5eda44b-ui-show-epic-and-depends-on-in-ticket-de"
created_at = "2026-04-01T21:56:10.584818Z"
updated_at = "2026-04-02T01:56:16.342780Z"
+++

## Spec

### Problem

The ticket detail panel (`apm-ui/src/components/TicketDetail.tsx`) renders core fields — title, state, effort, risk, priority — but has no awareness of `epic` or `depends_on`. Engineers cannot tell from the UI which epic a ticket belongs to, or which tickets it is waiting on before it can be dispatched.

The underlying `Frontmatter` struct in `apm-core/src/ticket.rs` does not yet declare `epic` or `depends_on` fields, so they are stripped during parsing even when present in the TOML frontmatter. Adding them to the struct is the minimal server-side change needed; the `TicketDetailResponse` already flattens `Frontmatter`, so the new fields will appear in the API automatically.

On the UI side, two small features are needed in the detail panel header: a clickable epic label (that sets the epic filter on the supervisor board so engineers can quickly scope to the same epic) and a dependency list (ticket IDs that link to each dep's detail panel, with strikethrough on resolved deps).

### Acceptance criteria

- [ ] When a ticket has no `epic` field, the detail panel shows no epic row
- [ ] When a ticket has an `epic` field, the detail panel shows a labelled row with the epic ID value
- [ ] Clicking the epic label sets `epicFilter` in the layout store to that epic ID
- [ ] When `epicFilter` is set in the layout store, the supervisor board hides tickets whose `epic` field does not match (tickets with no `epic` field are also hidden)
- [ ] Clicking the epic label a second time on the same ticket while the filter is already active clears the filter (toggle behaviour)
- [ ] When a ticket has no `depends_on` field (or an empty array), the detail panel shows no dependencies row
- [ ] When a ticket has a `depends_on` field, the detail panel lists each dep ticket ID
- [ ] Clicking a dep ticket ID in the detail panel sets `selectedTicketId` in the layout store to that dep's full ID, opening its detail panel
- [ ] Dep tickets that are absent from `blocking_deps` in the API response are shown with strikethrough text
- [ ] Dep tickets that appear in `blocking_deps` in the API response are shown without strikethrough
- [ ] A dep ticket ID that does not resolve to any known ticket renders as plain text (no link, no crash)
- [ ] Existing tickets without `epic` or `depends_on` in their frontmatter continue to load and display correctly

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
- Build a set of unresolved dep IDs: `const blockingSet = new Set((data.blocking_deps ?? []).map(d => d.id))`
- For each dep ID, look it up in the React Query cache via `useQueryClient().getQueryData<Ticket[]>(['tickets'])` to determine if it is a known ticket
- If found: render a `<button>` that calls `setSelectedTicketId(fullId)`. Apply `line-through` class if the dep ID is **absent** from `blockingSet` (i.e. it is resolved)
- If not found: render the raw ID as plain text (no crash)
- Use the existing `useQueryClient` import; do not add a new `useQuery` call

Resolution is determined entirely by `blocking_deps` — do not check state field names.

#### Tests

Add a unit test in `apm-core/src/ticket.rs` or `apm-core/tests/`: parse a ticket with `epic = "ab12cd34"` and `depends_on = ["cd56ef78"]` in frontmatter and assert both fields deserialize correctly. Parse a ticket without these fields and assert both are `None`. All existing tests must continue to pass.

### 1. `apm-core/src/ticket.rs` — extend `Frontmatter`

Add two optional fields to the `Frontmatter` struct:

```rust
pub epic: Option<String>,
pub depends_on: Option<Vec<String>>,
```

Both fields should be decorated with `#[serde(default, skip_serializing_if = "Option::is_none")]` (or equivalent) so existing ticket files without these fields continue to deserialize and round-trip correctly. No migration needed — TOML omits absent optional fields automatically.

`TicketDetailResponse` and `TicketResponse` in `apm-server/src/main.rs` both use `#[serde(flatten)] frontmatter`, so the new fields appear in all existing API responses with no server-side struct changes.

### 2. `apm-ui/src/store/useLayoutStore.ts` — add epic filter state

Add to `LayoutStore`:

```ts
epicFilter: string | null
setEpicFilter: (id: string | null) => void
```

Initialise `epicFilter: null`. The setter is a plain `set({ epicFilter: id })`.

### 3. `apm-ui/src/components/TicketDetail.tsx` — render the two new fields

**Interface update**: add `epic?: string` and `depends_on?: string[]` to the `TicketDetail` interface.

**Epic row** (render below the state badge / E-R-P row, conditional on `data.epic`):

- Label: "Epic"
- Value: a `<button>` showing the epic ID
- On click: if `epicFilter === data.epic`, call `setEpicFilter(null)`; otherwise call `setEpicFilter(data.epic)`
- Active state: highlight the button (e.g. blue border) when `epicFilter === data.epic`

**Depends on row** (render below epic row, conditional on `data.depends_on?.length`):

- Label: "Depends on"
- For each dep ID in the array, look it up in the `['tickets']` React Query cache (`useQueryClient().getQueryData<Ticket[]>(['tickets'])`) to get its current state
- If the dep resolves: render a `<button>` that calls `setSelectedTicketId(fullId)`. Apply `line-through` class when state is `implemented`, `accepted`, or `closed`
- If the dep does not resolve (not in cache): render the raw ID as plain text
- The tickets list query (`['tickets']`) is already populated by `SupervisorView`; no extra fetch needed in the detail component

Use `useQueryClient` (already imported) to read the cache; do not add a new `useQuery` call.

### 4. `apm-ui/src/components/supervisor/SupervisorView.tsx` — apply epic filter

Read `epicFilter` from the layout store:

```ts
const epicFilter = useLayoutStore((s) => s.epicFilter)
```

In the `columns` `useMemo`, after the existing `agentFilter` check, add:

```ts
if (epicFilter !== null) {
  filtered = filtered.filter((t) => t.epic === epicFilter)
}
```

Add `epicFilter` to the dependency array of the `useMemo`.

The `Ticket` type in `apm-ui/src/components/supervisor/types.ts` should gain `epic?: string` and `depends_on?: string[]` to match the extended API response.

### Order of changes

1. `apm-core/src/ticket.rs` — struct fields first (unblocks API)
2. `apm-ui/src/store/useLayoutStore.ts` — store additions
3. `apm-ui/src/components/supervisor/types.ts` — type extension
4. `apm-ui/src/components/supervisor/SupervisorView.tsx` — consume `epicFilter`
5. `apm-ui/src/components/TicketDetail.tsx` — render epic + depends_on rows

### Tests

- Unit test in `apm-core/src/ticket.rs` (or `apm-core/tests/`): parse a ticket with `epic = "ab12cd34"` and `depends_on = ["cd56ef78"]` in frontmatter; assert fields deserialize correctly. Parse a ticket without these fields; assert fields are `None`.
- Existing integration tests must continue to pass (`cargo test --workspace`).

### Open questions


### Amendment requests

- [x] The strikethrough condition "state is `implemented`, `accepted`, or `closed`" must not hardcode state names. The resolution check belongs server-side: `blocking_deps` (introduced by da95246d) already captures only unresolved deps using `satisfies_deps`/`terminal`. In the UI, a dep is "resolved" (strikethrough) when it does NOT appear in `blocking_deps`. Remove the state name list from the AC and Approach; replace with "dep is considered resolved when it is absent from `blocking_deps` in the API response".

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
