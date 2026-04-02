+++
id = "711e717e"
title = "UI: add epic filter to supervisor board filter bar"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "66037"
branch = "ticket/711e717e-ui-add-epic-filter-to-supervisor-board-f"
created_at = "2026-04-01T21:56:24.806901Z"
updated_at = "2026-04-02T00:57:35.927094Z"
+++

## Spec

### Problem

The supervisor board filter bar has state and agent filters but no epic filter. When multiple epics are active, all their tickets appear together, making it impossible for the supervisor to isolate a single epic's work in the board view.

The desired behaviour: an epic dropdown in the filter bar (beside the existing state and agent dropdowns) lets the supervisor select one epic and hide all tickets that belong to other epics, or select "All" to restore the default view. The dropdown is populated from `GET /api/epics`.

That API route does not yet exist. This ticket adds both the server-side endpoint (minimal: branch scan + name parsing, no ticket counts) and the UI dropdown.

### Acceptance criteria

- [ ] An "Epic" dropdown appears in the supervisor board filter bar, positioned after the "All agents" dropdown
- [ ] The dropdown contains an "All epics" option that is selected by default and shows all tickets
- [ ] The dropdown options are populated from `GET /api/epics` and show each epic's title
- [ ] Selecting an epic hides all ticket cards whose `epic` field does not match the selected epic id
- [ ] Tickets with no `epic` field are hidden when any specific epic is selected
- [ ] Selecting "All epics" after a specific epic restores the full board view
- [ ] The "No tickets match the current filters" empty state appears when an epic is selected and no tickets match
- [ ] The epic filter composes with the existing state, agent, and search filters (all active simultaneously)
- [ ] `GET /api/epics` returns a JSON array; each element has `id`, `title`, and `branch` string fields
- [ ] `GET /api/epics` returns an empty array when no `epic/*` branches exist
- [ ] The dropdown renders but shows only "All epics" when `GET /api/epics` returns an empty array

### Out of scope

- Epic filter in the Queue panel (separate item in docs/epics.md UI section)
- Epic column in the Queue panel
- Epic selector in Engine controls
- POST /api/epics (create epic)
- GET /api/epics/:id (epic detail with ticket list)
- Ticket lock icon for unresolved `depends_on` entries
- Clickable epic label in Ticket detail panel
- Derived epic state or ticket counts in the `GET /api/epics` response
- `epic` and `target_branch` fields on `CreateTicketRequest`
- Any changes to `apm work --epic` or the work engine epic filter

### Approach

Four files change. Order: Rust core, server, UI types, UI component.

**1. `apm-core/src/git.rs`** — add `epic_branches`

Add a public function after `ticket_branches` (line 65), following the identical pattern: scan `git branch --list epic/*` for local branches and `git branch -r --list origin/epic/*` for remote ones, de-duplicating by name.

**2. `apm-core/src/ticket.rs`** — add `epic` to `Frontmatter`

Add after the `branch` field (line 50):

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub epic: Option<String>,
```

Because `TicketResponse` uses `#[serde(flatten)] frontmatter`, the `epic` field appears automatically in all existing ticket API responses — no struct changes in the server needed.

**3. `apm-server/src/main.rs`** — add `GET /api/epics`

Add `EpicSummary` struct near the top (after `TicketDetailResponse`):

```rust
#[derive(serde::Serialize)]
struct EpicSummary {
    id: String,
    title: String,
    branch: String,
}
```

Branch name format: `epic/<8-char-id>-<slug>`. Parsing: strip `epic/` prefix, take first 8 chars as `id`, take the remainder after position 9 (skipping the hyphen separator) as slug, replace hyphens with spaces for `title`.

Handler (returns empty array when no git root, so in-memory mode works fine):

```rust
async fn list_epics(State(state): State<Arc<AppState>>) -> Response {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Json(Vec::<EpicSummary>::new()).into_response(),
    };
    let branches = tokio::task::spawn_blocking(move || {
        apm_core::git::epic_branches(&root).unwrap_or_default()
    }).await.unwrap_or_default();
    let epics: Vec<EpicSummary> = branches.into_iter().filter_map(|b| {
        let slug_part = b.strip_prefix("epic/")?;
        let id = slug_part.get(..8)?.to_string();
        let title_slug = slug_part.get(9..).unwrap_or("");
        let title = title_slug.replace('-', " ");
        Some(EpicSummary { id, title, branch: b })
    }).collect();
    Json(epics).into_response()
}
```

Register `.route("/api/epics", get(list_epics))` in both router branches (~line 667 and ~line 694).

**4. `apm-ui/src/components/supervisor/types.ts`** — extend `Ticket`

Add one optional field: `epic?: string`

**5. `apm-ui/src/components/supervisor/SupervisorView.tsx`** — add epic filter

a) Add `Epic` type and `fetchEpics` after `fetchTickets`:

```typescript
interface Epic { id: string; title: string; branch: string }
async function fetchEpics(): Promise<Epic[]> {
  const res = await fetch('/api/epics')
  if (!res.ok) return []
  return res.json()
}
```

b) Inside `SupervisorView`, add state and query after the existing `agentFilter` line:

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

Add `epicFilter` to the dependency array.

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

**Tests**: `cargo test --workspace` must pass. The branch-scanning logic in `epic_branches` is trivially derived from `ticket_branches`; no new test required. If a test is added, use the `git_setup` helper already present in `apm-server/src/main.rs`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:57Z | groomed | in_design | philippepascal |