+++
id = "54b043f7"
title = "Add /api/epics server routes"
state = "closed"
priority = 4
effort = 5
risk = 3
author = "claude-0401-2145-a8f3"
agent = "37189"
branch = "ticket/54b043f7-add-api-epics-server-routes"
created_at = "2026-04-01T21:55:53.796830Z"
updated_at = "2026-04-02T19:06:46.580994Z"
+++

## Spec

### Problem

The apm-server has no API routes for epics. Clients (UI, external tooling) cannot list all epics, create a new epic, or inspect a single epic with its associated tickets.

Three routes are specified in `docs/epics.md` (§ apm-server changes — New routes):

- `GET /api/epics` — list all epics discovered from `epic/*` git branches, with derived state and per-state ticket counts
- `POST /api/epics` — create a new epic branch (`epic/<id>-<slug>`) from the tip of `origin/main`, seed it with a minimal `EPIC.md`, and push it
- `GET /api/epics/:id` — return a single epic with the same summary fields plus a full `tickets` array

Epic state is derived on demand from the states of associated tickets (those whose frontmatter contains `epic = "<id>"`). That field does not yet exist on `Frontmatter`; it must be added as a prerequisite.

### Acceptance criteria

- [x] `GET /api/epics` returns `[]` when no `epic/*` branches exist locally or at origin
- [x] `GET /api/epics` returns one `EpicSummary` entry per `epic/*` branch found (local or `origin/*`)
- [x] Each `EpicSummary` contains `id`, `title`, `branch`, `state`, and `ticket_counts` fields
- [x] `GET /api/epics` on an in-memory server returns HTTP 501
- [x] Epic `state` is `"empty"` when no tickets reference the epic (i.e. no ticket frontmatter carries `epic = "<id>"`)
- [x] Epic `state` is `"active"` when any associated ticket is in a state whose `StateConfig.actionable` contains `"agent"`
- [x] Epic `state` is `"complete"` when all associated tickets are in states where `satisfies_deps = true` or `terminal = true`, and at least one ticket is in a state where `satisfies_deps = true`
- [x] Epic `state` is `"done"` when all associated tickets are in states where `terminal = true`
- [x] `POST /api/epics` with `{"title": "My Epic"}` returns HTTP 201 with a new `EpicSummary` (state `"empty"`, empty `ticket_counts`)
- [x] After `POST /api/epics`, an `epic/<id>-<slug>` branch exists at origin
- [x] `POST /api/epics` with missing or empty `title` returns HTTP 400
- [x] `POST /api/epics` on an in-memory server returns HTTP 501
- [x] `GET /api/epics/:id` returns the matching epic with all `EpicSummary` fields plus a `tickets` array
- [x] Each entry in `tickets` uses the same shape as `TicketResponse` (flattened frontmatter + `body`, `has_open_questions`, `has_pending_amendments`)
- [x] `GET /api/epics/:id` returns HTTP 404 when no `epic/*` branch whose ID segment matches `/:id` exists
- [x] `GET /api/epics/:id` on an in-memory server returns HTTP 501

### Out of scope

- `apm epic` CLI subcommands (`epic new`, `epic list`, `epic show`, `epic close`) — CLI is a separate concern
- Adding `epic`, `target_branch`, `depends_on` to `POST /api/tickets` / `CreateTicketRequest`
- Work engine exclusive-epic scheduling (`POST /api/work/start` with `epic` field)
- UI changes (queue epic column, supervisor board filter, engine epic selector)
- `depends_on` scheduling enforcement in the work engine
- `apm new --epic` CLI flag

### Approach

Five files change in order.

**0. `apm-core/src/config.rs` — add `satisfies_deps` flag to `StateConfig`**

Add to `StateConfig`:

```rust
#[serde(default)]
pub satisfies_deps: bool,
```

This flag marks states where a ticket is considered "done enough" to unblock dependents (e.g. `implemented`). Defaults to `false`. The canonical `apm.toml` must set `satisfies_deps = true` on the `implemented` state.

**1. `apm-core/src/ticket.rs` — add three optional frontmatter fields**

Add to `Frontmatter` (all with `#[serde(skip_serializing_if = "Option::is_none")]`):

```rust
pub epic: Option<String>,
pub target_branch: Option<String>,
pub depends_on: Option<Vec<String>>,
```

`epic` is required for filtering tickets by epic. `target_branch` and `depends_on` are included now so all existing ticket routes expose them automatically through `#[serde(flatten)]` — no struct changes to `TicketResponse` or `TicketDetailResponse` needed.

**2. `apm-core/src/git.rs` — two new public functions**

`pub fn epic_branches(root: &Path) -> Result<Vec<String>>` — mirrors `ticket_branches` but matches `epic/*` and `origin/epic/*` patterns. Deduplicates local and remote entries the same way.

`pub fn create_epic_branch(root: &Path, title: &str) -> Result<(String, String)>` — returns `(id, branch_name)`:
1. `gen_hex_id()` for the 8-char ID
2. `crate::ticket::slugify(title)` for the slug
3. `branch = format!("epic/{id}-{slug}")`
4. Best-effort `run(root, &["fetch", "origin", "main"])` — ignore error so tests without remotes still pass
5. `run(root, &["branch", &branch, "origin/main"])` — create local branch at remote main tip; fall back to `run(root, &["branch", &branch, "main"])` if that fails
6. `commit_to_branch(root, &branch, "EPIC.md", &format!("# {title}\n"), "epic: init")`
7. Best-effort `push_branch(root, &branch)`
8. Return `(id, branch)`

**3. `apm-server/src/main.rs` — structs, helpers, handlers, routes**

New structs (near the other response/request types):

```rust
#[derive(serde::Serialize)]
struct EpicSummary {
    id: String,
    title: String,
    branch: String,
    state: String,
    ticket_counts: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
struct EpicDetailResponse {
    #[serde(flatten)]
    summary: EpicSummary,
    tickets: Vec<TicketResponse>,
}

#[derive(serde::Deserialize)]
struct CreateEpicRequest {
    title: Option<String>,
}
```

`parse_epic_branch(branch: &str) -> Option<(String, String)>` — strips `epic/` prefix, splits on first `-`, title-cases the slug. Returns `None` for malformed names.

`derive_epic_state(tickets: &[&Ticket], states: &[apm_core::config::StateConfig]) -> String` — config-driven; no hardcoded state ID strings:
1. Build a `HashMap<&str, &StateConfig>` keyed by `state.id`
2. Empty ticket slice → `"empty"`
3. Any ticket in a state where `actionable` contains `"agent"` → `"active"`
4. All tickets in states where `satisfies_deps || terminal`, and at least one `satisfies_deps` → `"complete"`
5. All tickets in states where `terminal` → `"done"`
6. Otherwise → `"active"`

`build_epic_summary(branch: &str, all_tickets: &[Ticket], states: &[StateConfig]) -> Option<EpicSummary>` — calls `parse_epic_branch`, filters tickets by `frontmatter.epic`, counts states, calls `derive_epic_state(tickets, states)`, returns the summary.

`list_epics`: guard 501 if in-memory; load config via `Config::load(&root)`; `spawn_blocking` → `epic_branches`; `load_tickets`; map branches through `build_epic_summary`; return vec.

`create_epic`: guard 501 if in-memory; validate title non-empty → 400; `spawn_blocking` → `create_epic_branch`; return 201 with `EpicSummary` (empty counts, state `"empty"`).

`get_epic`: guard 501 if in-memory; load config; `spawn_blocking` → `epic_branches`; find branch matching `/:id` → 404 if absent; `load_tickets` filtered by epic id; return `EpicDetailResponse`.

Route registration — add to both `build_app` and `build_app_with_tickets`:

```
.route("/api/epics", get(list_epics).post(create_epic))
.route("/api/epics/:id", get(get_epic))
```

**4. Tests (inline `#[cfg(test)]` in `apm-server/src/main.rs`)**

Required: `list_epics_in_memory_returns_501`, `create_epic_missing_title_returns_400`, `create_epic_empty_title_returns_400`, `create_epic_in_memory_returns_501`, `get_epic_in_memory_returns_501`.

Round-trip tests (create → list → get) may use the existing temp-repo helpers already present in the test module.

### Open questions


### Amendment requests

- [x] Delete the duplicate helper sections at the bottom of Approach that still contain the old `derive_epic_state` signature and implementation with hardcoded state names ("in_design", "in_progress", "accepted", "closed", "implemented"). The corrected Approach at the top is authoritative; the entire old duplicate block below must be removed.
- [x] The Acceptance criteria and `derive_epic_state` steps in the Approach use `"in_progress"` and `"implemented"` as epic state labels — these are ticket workflow state IDs being hardcoded as epic vocabulary. Replace throughout: `"in_progress"` → `"active"`, `"implemented"` → `"complete"`. This applies to AC items, the return values in `derive_epic_state` steps 3, 4, and 6, and any output examples. The epic state labels must match what ticket a5e1ea24 defines: `empty`, `active`, `done`, `complete`.
- [x] Remove the sentence in step 4 of the Approach that says "Find the `[[workflow.states]]` entry with `id = "implemented"` and add `satisfies_deps = true`" — this references a state by hardcoded name and is not this ticket's responsibility. That config change is owned by the workflow setup.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:51Z | groomed | in_design | philippepascal |
| 2026-04-02T00:57Z | in_design | specd | claude-0402-0055-spec1 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:42Z | ammend | in_design | philippepascal |
| 2026-04-02T01:45Z | in_design | specd | claude-0402-0200-spec2 |
| 2026-04-02T01:55Z | specd | ammend | philippepascal |
| 2026-04-02T01:56Z | ammend | in_design | philippepascal |
| 2026-04-02T01:59Z | in_design | specd | claude-0402-0210-spec3 |
| 2026-04-02T02:03Z | specd | ammend | apm |
| 2026-04-02T02:11Z | ammend | in_design | philippepascal |
| 2026-04-02T02:12Z | in_design | specd | claude-0402-0215-spec4 |
| 2026-04-02T02:21Z | specd | ammend | apm |
| 2026-04-02T02:21Z | ammend | in_design | philippepascal |
| 2026-04-02T02:25Z | in_design | specd | claude-0402-0230-spec5 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T06:30Z | ready | in_progress | philippepascal |
| 2026-04-02T06:34Z | in_progress | implemented | claude-0402-0630-impl1 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |