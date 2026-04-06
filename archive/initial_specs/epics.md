# Epics — Design Spec

## Concept

An epic is a git branch (`epic/<id>-<slug>`). That's it — no separate file format, no new struct. The branch IS the epic. Tickets associated with an epic branch from it instead of `main`, and their PRs target it. When all tickets are done, the epic branch is merged to `main` as one coherent unit.

The title and ID are embedded in the branch name. Epic state is derived from ticket states. No frontmatter file is needed.

An optional `EPIC.md` prose file on the epic branch can hold a free-text description, but it is not parsed by APM — it is purely for humans reading the branch on GitHub.

---

## Data model

### Epic identity

```
epic/<8-char-id>-<slug>
```

Example: `epic/ab12cd34-user-authentication`

- **ID**: first 8 characters of a generated UUID (same scheme as ticket IDs)
- **Title**: reconstructed from the slug (hyphens → spaces, title-cased)
- **State**: derived from ticket states (see below)
- **Tickets**: all tickets whose frontmatter contains `epic = "ab12cd34"`

### Derived epic state

| Condition | Derived state |
|-----------|--------------|
| No tickets yet | `empty` |
| Any ticket is `in_design` or `in_progress` | `in_progress` |
| All tickets are `implemented` or later | `implemented` |
| All tickets are `accepted` or `closed` | `done` |
| Otherwise | `in_progress` |

Epic state is never written anywhere — it is computed on demand.

### Ticket frontmatter additions

Three new optional fields:

```toml
epic         = "ab12cd34"                          # short epic ID
target_branch = "epic/ab12cd34-user-authentication" # PR and worktree target
depends_on   = ["cd56ef78", "12ab34cd"]            # ticket IDs that must be implemented first
```

- `epic` and `target_branch` are set together when a ticket is created inside an epic
- `depends_on` can be set on any ticket, epic or not
- All three fields are optional; omitting them preserves current behaviour exactly

---

## Commands

### `apm epic new <title>`

1. Generate a short ID (8 hex chars)
2. Slugify the title
3. `git fetch origin main`
4. `git checkout -b epic/<id>-<slug>` from `origin/main` HEAD
5. Optionally create `EPIC.md` with the title as H1 and commit it (one small commit establishes the branch as diverged from main, making it visible in `git log --oneline main..epic/...`)
6. `git push -u origin epic/<id>-<slug>`
7. Print the branch name

### `apm epic list`

List all `epic/*` remote branches. For each, show:
- Short ID and title (from branch name)
- Derived state
- Ticket counts by state (e.g. `2 in_progress, 1 ready, 3 implemented`)

Reads: `git branch -r | grep 'epic/'`, then scans ticket frontmatter.

### `apm epic show <id>`

Show:
- Title, branch, derived state
- Table of tickets: ID, title, state, assignee, depends_on

### `apm new --epic <id> [--depends-on <ticket-id>,...]`

Create a ticket with `epic`, `target_branch`, and optionally `depends_on` pre-filled. The ticket branch is created from the epic branch tip, not from `main`.

### `apm epic close <id>`

Create a PR from the epic branch to `main` (using `gh pr create`). Does not merge — merging requires human approval as usual.

---

## Workflow integration

### `apm start <id>`

If `target_branch` is set in the ticket frontmatter, provision the worktree from `target_branch` instead of the default branch. The ticket's own branch (`ticket/<id>-<slug>`) is created from the tip of `target_branch`.

No change needed if `target_branch` is absent — existing behaviour is preserved exactly.

### Completion strategy

`gh_pr_create_or_update` already takes a `default_branch` argument. Change the call site to pass `ticket.frontmatter.target_branch.as_deref().unwrap_or(default_branch)` instead of `default_branch` directly.

One line change.

### `apm sync`

No change needed. `apm sync` already scans all `ticket/*` branches for ticket files. Epic branches are not scanned (they carry no ticket files). `apm epic list` and `apm epic show` do their own branch scanning.

---

## `depends_on` scheduling

`depends_on = ["cd56ef78"]` in ticket frontmatter means: do not dispatch this ticket until `cd56ef78` is in state `implemented` or later.

### Engine loop change

In `run_engine_loop` / `spawn_next_worker`, before dispatching a candidate ticket:

```rust
let blocked = ticket.frontmatter.depends_on.iter().any(|dep_id| {
    tickets.iter()
        .find(|t| t.frontmatter.id.starts_with(dep_id))
        .map(|t| !is_implemented_or_later(&t.frontmatter.state, &config))
        .unwrap_or(false)  // unknown dep → not blocking
});
if blocked { continue; }
```

`is_implemented_or_later` checks whether the state has `terminal = true` or appears after `implemented` in the workflow states list — config-driven, not hardcoded.

`depends_on` works for any ticket, epic or not. It is just more commonly useful within an epic.

### UI

Ticket cards with unresolved `depends_on` entries show a small lock icon. The queue panel tooltip on hover lists the blocking ticket IDs. No other UI change needed.

---

## `apm work` — epic scheduling

### Open mode (default, unchanged)

All actionable tickets compete by priority score. Epic membership is irrelevant.

### Exclusive mode

```
apm work --epic <id>
```

The engine only dispatches tickets where `frontmatter.epic == id`. Free tickets are ignored. Dependency ordering applies within the epic.

No other modes (balanced, `--and-free`, per-epic limits) — cut for simplicity.

### Config shorthand

```toml
[work]
epic = "ab12cd34"   # if set, implies exclusive mode
```

---

## UI additions

All minimal:

**Queue panel**: add an "Epic" column showing the short epic ID or "—". Add an epic filter dropdown.

**Supervisor board**: add epic filter dropdown to the existing filter bar (already has state + agent filters).

**Engine controls**: add an optional epic selector before starting. When exclusive mode is active, show a label: "epic: user-auth".

No epic overview panel — `apm epic list` in the terminal covers this, and the supervisor board filter covers the visual case.

---

## apm-server changes

### New routes

```
GET  /api/epics              → list all epics (branch scan + derived state)
POST /api/epics              → create a new epic (runs apm epic new)
GET  /api/epics/:id          → single epic with ticket list
```

Response shape for a single epic:

```json
{
  "id": "ab12cd34",
  "title": "User Authentication",
  "branch": "epic/ab12cd34-user-authentication",
  "state": "in_progress",
  "ticket_counts": { "in_progress": 2, "ready": 1, "implemented": 3 }
}
```

Epic list response is `[EpicSummary]`. Epic detail response adds a `tickets` array (same `TicketResponse` shape as existing ticket routes).

### Ticket routes — no struct changes needed

`TicketResponse` and `TicketDetailResponse` both use `#[serde(flatten)] frontmatter`. The new frontmatter fields (`epic`, `target_branch`, `depends_on`) appear automatically in all existing ticket API responses — no struct changes required.

### CreateTicketRequest — two new optional fields

```rust
pub struct CreateTicketRequest {
    pub title: String,
    // existing fields ...
    pub epic: Option<String>,          // short epic ID
    pub depends_on: Option<Vec<String>>,
}
```

When `epic` is set, the server resolves `target_branch` from the epic branch name before calling `apm new`.

### Work engine — epic filter

`POST /api/work/start` body gains one optional field:

```json
{ "max_workers": 3, "epic": "ab12cd34" }
```

When `epic` is set, `run_engine_loop` filters candidates to `frontmatter.epic == id` before the existing priority sort. `GET /api/work/status` response includes `"epic": "ab12cd34"` when exclusive mode is active (null otherwise).

---

## apm-ui changes

### New ticket modal

Add two optional fields below the title input:

- **Epic** — dropdown populated from `GET /api/epics`; selecting one pre-fills the epic ID. Omitting it creates a free ticket (existing behaviour).
- **Depends on** — multi-value text input for ticket IDs; stored as `depends_on` array.

### Ticket detail panel

Show `epic` and `depends_on` when present:

- **Epic**: clickable label that sets the epic filter on the supervisor board.
- **Depends on**: list of ticket IDs; each links to that ticket's detail panel. Resolved tickets (implemented+) shown with strikethrough.

### Ticket cards (queue and supervisor board)

Cards where `depends_on` has at least one unresolved entry show a small lock icon. Tooltip lists the blocking ticket IDs and their current states.

### Queue panel

Add an **Epic** column showing the short epic ID or "—". Add an epic filter dropdown (same pattern as the existing state filter).

### Supervisor board

Add an epic filter dropdown to the existing filter bar. Selecting an epic hides tickets from other epics. Selecting "All" restores the default view.

### Engine controls

Add an optional **Epic** selector (dropdown from `GET /api/epics`) before starting the engine. When exclusive mode is active, show a small label: `epic: user-auth`. The label links to the epic filter on the supervisor board.

---

## What is explicitly not supported

- **Moving a ticket into an epic after work has started** — refused. Create the ticket with `--epic` from the beginning, or not at all.
- **Moving a ticket out of an epic** — not supported. Close and recreate outside the epic if needed.
- **Nested epics** — not supported.
- **Automatic merging** — `apm epic close` opens a PR; a human merges it.
- **`apm epic sync`** — run `git merge main` on the epic branch manually when needed.
- **Balanced / multi-epic concurrent scheduling** — open or exclusive only.
- **Epic state transitions** — state is always derived; never written.
