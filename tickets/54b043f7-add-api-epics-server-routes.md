+++
id = "54b043f7"
title = "Add /api/epics server routes"
state = "in_design"
priority = 4
effort = 4
risk = 3
author = "claude-0401-2145-a8f3"
agent = "35861"
branch = "ticket/54b043f7-add-api-epics-server-routes"
created_at = "2026-04-01T21:55:53.796830Z"
updated_at = "2026-04-02T01:42:05.088575Z"
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

- [ ] `GET /api/epics` returns `[]` when no `epic/*` branches exist locally or at origin
- [ ] `GET /api/epics` returns one `EpicSummary` entry per `epic/*` branch found (local or `origin/*`)
- [ ] Each `EpicSummary` contains `id`, `title`, `branch`, `state`, and `ticket_counts` fields
- [ ] `GET /api/epics` on an in-memory server returns HTTP 501
- [ ] Epic `state` is `"empty"` when no tickets reference the epic (i.e. no ticket frontmatter carries `epic = "<id>"`)
- [ ] Epic `state` is `"in_progress"` when any associated ticket is in a state whose `StateConfig.actionable` contains `"agent"`
- [ ] Epic `state` is `"implemented"` when all associated tickets are in states where `satisfies_deps = true` or `terminal = true`, and at least one ticket is in a state where `satisfies_deps = true`
- [ ] Epic `state` is `"done"` when all associated tickets are in states where `terminal = true`
- [ ] `POST /api/epics` with `{"title": "My Epic"}` returns HTTP 201 with a new `EpicSummary` (state `"empty"`, empty `ticket_counts`)
- [ ] After `POST /api/epics`, an `epic/<id>-<slug>` branch exists at origin
- [ ] `POST /api/epics` with missing or empty `title` returns HTTP 400
- [ ] `POST /api/epics` on an in-memory server returns HTTP 501
- [ ] `GET /api/epics/:id` returns the matching epic with all `EpicSummary` fields plus a `tickets` array
- [ ] Each entry in `tickets` uses the same shape as `TicketResponse` (flattened frontmatter + `body`, `has_open_questions`, `has_pending_amendments`)
- [ ] `GET /api/epics/:id` returns HTTP 404 when no `epic/*` branch whose ID segment matches `/:id` exists
- [ ] `GET /api/epics/:id` on an in-memory server returns HTTP 501

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
3. Any ticket in a state where `actionable` contains `"agent"` → `"in_progress"`
4. All tickets in states where `satisfies_deps || terminal`, and at least one `satisfies_deps` → `"implemented"`
5. All tickets in states where `terminal` → `"done"`
6. Otherwise → `"in_progress"`

`build_epic_summary(branch: &str, all_tickets: &[Ticket], states: &[StateConfig]) -> Option<EpicSummary>` — calls `parse_epic_branch`, filters tickets by `frontmatter.epic`, counts states, calls `derive_epic_state(tickets, states)`, returns the summary.

`list_epics`: guard 501 if in-memory; load config via `Config::load(&root)`; `spawn_blocking` → `epic_branches`; `load_tickets`; map branches through `build_epic_summary`; return vec.

`create_epic`: guard 501 if in-memory; validate title non-empty → 400; `spawn_blocking` → `create_epic_branch`; return 201 with `EpicSummary` (empty counts, state `"empty"`).

`get_epic`: guard 501 if in-memory; load config; `spawn_blocking` → `epic_branches`; find branch matching `/:id` → 404 if absent; `load_tickets` filtered by epic id; return `EpicDetailResponse`.

Route registration — add to both `build_app` and `build_app_with_tickets`:

```
.route("/api/epics", get(list_epics).post(create_epic))
.route("/api/epics/:id", get(get_epic))
```

**4. `apm.toml` — set `satisfies_deps = true` on the `implemented` state**

Find the `[[workflow.states]]` entry with `id = "implemented"` and add `satisfies_deps = true`. This is required for `derive_epic_state` to correctly identify the `"implemented"` epic state.

**5. Tests (inline `#[cfg(test)]` in `apm-server/src/main.rs`)**

Required: `list_epics_in_memory_returns_501`, `create_epic_missing_title_returns_400`, `create_epic_empty_title_returns_400`, `create_epic_in_memory_returns_501`, `get_epic_in_memory_returns_501`.

Round-trip tests (create → list → get) may use the existing temp-repo helpers already present in the test module.

### 1. `apm-core/src/ticket.rs` — add three optional frontmatter fields

Add to `Frontmatter` (all with `#[serde(skip_serializing_if = "Option::is_none")]`):

```rust
pub epic: Option<String>,
pub target_branch: Option<String>,
pub depends_on: Option<Vec<String>>,
```

The `epic` field is the minimum required for filtering tickets by epic. The other two are added now so all ticket routes expose them automatically via `#[serde(flatten)]` (no extra work needed per the design doc).

### 2. `apm-core/src/git.rs` — two new public functions

**`pub fn epic_branches(root: &Path) -> Result<Vec<String>>`**

Mirror `ticket_branches` but use `epic/*` / `origin/epic/*` patterns. Deduplicate local and remote entries the same way.

**`pub fn create_epic_branch(root: &Path, title: &str) -> Result<(String, String)>`**

Returns `(id, branch_name)`.

1. Call `gen_hex_id()` for the 8-char ID
2. Call `crate::ticket::slugify(title)` for the slug
3. Compose `branch = format!("epic/{id}-{slug}")`
4. Best-effort `run(root, &["fetch", "origin", "main"])` (ignore error — offline repos must still work in tests)
5. `run(root, &["branch", &branch, "origin/main"])` — create the local branch ref at remote main's tip; if that fails (no remote), fall back to `run(root, &["branch", &branch, "main"])`
6. `commit_to_branch(root, &branch, "EPIC.md", &format!("# {title}\n"), "epic: init")`
7. Best-effort `push_branch(root, &branch)`
8. Return `(id, branch)`

### 3. `apm-server/src/main.rs` — new structs, helpers, handlers, and route registrations

**Structs** (add near the other response/request structs):

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

**Helper: `parse_epic_branch(branch: &str) -> Option<(String, String)>`**

- Strip the `epic/` prefix
- Split on the first `-` to separate `id` (8 chars) from `slug`
- Convert slug to title: replace `-` with space, title-case each word
- Return `Some((id, title))`; return `None` for malformed branch names

**Helper: `derive_epic_state(tickets: &[&apm_core::ticket::Ticket]) -> String`**

Implements the table from `docs/epics.md`:
- `tickets` is empty → `"empty"`
- Any ticket state is `in_design` or `in_progress` → `"in_progress"`
- All ticket states are in `{"accepted", "closed"}` → `"done"`
- All ticket states are in `{"implemented", "accepted", "closed"}` → `"implemented"`
- Otherwise → `"in_progress"`

**Helper: `build_epic_summary(branch: &str, all_tickets: &[apm_core::ticket::Ticket]) -> Option<EpicSummary>`**

- Call `parse_epic_branch` — return `None` on failure
- Filter `all_tickets` to those where `frontmatter.epic.as_deref() == Some(id)`
- Build `ticket_counts: HashMap<String, usize>` by counting each state
- Call `derive_epic_state`
- Return `Some(EpicSummary { ... })`

**Handler: `list_epics`**

1. Guard: return 501 if `state.git_root()` is `None`
2. `spawn_blocking`: call `apm_core::git::epic_branches(root)`
3. `load_tickets` (reuse existing helper)
4. For each branch, call `build_epic_summary`; collect non-None results
5. Return `Json(summaries)`

**Handler: `create_epic`**

1. Guard: 501 if in-memory
2. Validate title is non-empty — return 400 if not
3. `spawn_blocking`: call `apm_core::git::create_epic_branch(root, &title)` → `(id, branch)`
4. Build `EpicSummary` with empty `ticket_counts` and state `"empty"`
5. Return `(StatusCode::CREATED, Json(summary))`

**Handler: `get_epic`**

1. Guard: 501 if in-memory
2. `spawn_blocking`: call `apm_core::git::epic_branches(root)`; find the branch whose `epic/<id>-` prefix matches; return 404 if not found
3. `load_tickets`; filter to those where `frontmatter.epic.as_deref() == Some(&id)`
4. Call `build_epic_summary` for the branch
5. Build `TicketResponse` for each matched ticket (same as `list_tickets`)
6. Return `Json(EpicDetailResponse { summary, tickets })`

**Route registration** — add to both `build_app` and `build_app_with_tickets` Router chains:

```
.route("/api/epics", get(list_epics).post(create_epic))
.route("/api/epics/:id", get(get_epic))
```

### 4. Tests (inline in `apm-server/src/main.rs` `#[cfg(test)]` block)

Add unit tests covering at minimum:
- `list_epics_in_memory_returns_501`
- `create_epic_missing_title_returns_400`
- `create_epic_empty_title_returns_400`
- `create_epic_in_memory_returns_501`
- `get_epic_in_memory_returns_501`

Full round-trip tests (branch creation + list + get) require a real git repo; use the existing temp-repo test helpers if the pattern is already established in `apm-server` tests.

### Open questions


### Amendment requests

- [x] `derive_epic_state` in the Approach hardcodes "in_design", "in_progress", "accepted", "closed", "implemented". Replace with config-driven logic: the server loads workflow config and passes `&config.workflow.states` to the helper; the helper uses `actionable`, `satisfies_deps`, and `terminal` flags (same rules as a5e1ea24). No state ID string comparisons.
- [ ] Update the epic `state` AC items to describe the four outcomes using config flags, not state names.

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