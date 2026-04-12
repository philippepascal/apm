+++
id = "7bb8eacb"
title = "Extract ticket handlers from main.rs into handlers/tickets.rs"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7bb8eacb-extract-ticket-handlers-from-main-rs-int"
created_at = "2026-04-12T09:03:09.497239Z"
updated_at = "2026-04-12T17:11:09.319204Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["a6bc1326", "2973f8d1"]
+++

## Spec

### Problem

main.rs in apm-server currently contains roughly 500 lines of ticket-related HTTP handler code — request/response structs, helper functions, and eight async handler functions — all mixed in with server bootstrap, auth routes, and worker-queue code. This makes the file very long (~4000+ lines) and hard to navigate.\n\nThe desired state is that all ticket CRUD logic lives in a dedicated handlers/tickets.rs module. After extraction, main.rs retains only route registration (referencing handlers by path) and the cross-cutting infrastructure types (AppState, AppError, server startup). This mirrors the existing pattern for other logical groupings in the codebase (agents.rs, auth.rs, workers.rs, etc.).\n\nThis ticket depends on two prerequisite refactors (a6bc1326 and 2973f8d1) that move DTOs and core business logic out of main.rs, so by the time this work runs, the ticket handler code is already relatively self-contained.

### Acceptance criteria

- [x] `cargo build -p apm-server` succeeds after the extraction with no compiler errors or warnings
- [x] `cargo test -p apm-server` passes (all existing tests continue to pass)
- [x] `apm-server/src/handlers/tickets.rs` exists and contains all eight handler functions: `list_tickets`, `get_ticket`, `transition_ticket`, `put_body`, `patch_ticket`, `create_ticket`, `batch_transition`, `batch_priority`
- [x] `apm-server/src/handlers/tickets.rs` contains the handler-private helper functions: `extract_section`, `extract_frontmatter_raw`, `extract_history_raw`, `load_tickets`
- [x] `apm-server/src/handlers/mod.rs` exists and declares `pub mod tickets`
- [x] `main.rs` imports handlers via `mod handlers` and references ticket handlers from `handlers::tickets`
- [x] `main.rs` no longer directly defines any of the moved functions (grep for their definition sites yields zero results in main.rs)
- [x] `AppError` and `AppState` remain defined in `main.rs`; `handlers/tickets.rs` imports them from `crate`
- [x] `handlers/tickets.rs` imports ticket DTOs via `use crate::models::*` (not re-defining them)
- [x] `handlers/tickets.rs` calls `compute_blocking_deps` and `compute_valid_transitions` from `apm_core` (not defining them locally)
- [x] The HTTP routes registered in `build_app()` are unchanged — same verbs, same paths, same handler function bindings

### Out of scope

- Extracting non-ticket handlers (auth, agents, workers, login) — those are separate refactor tickets
- Renaming any handler function, struct, or route path
- Changing any handler's logic or behavior
- Moving `AppError` or `AppState` out of main.rs
- Adding new ticket endpoints or fields
- Extracting spec-section handlers (get_ticket_spec, update_ticket_spec) if they were already moved by a prerequisite ticket — verify first and skip if already gone
- Writing tests that do not already exist
- Moving ticket DTOs (TransitionOption, TicketResponse, TicketsEnvelope, BlockingDep, TicketDetailResponse, TransitionRequest, BatchTransitionRequest, BatchPriorityRequest, PutBodyRequest, PatchTicketRequest, CreateTicketRequest, ListTicketsQuery, BatchFailure, BatchResult) — prerequisite a6bc1326 already moves these to `models.rs`
- Moving `compute_blocking_deps` or `compute_valid_transitions` — prerequisite 2973f8d1 already moves these to `apm_core`

### Approach

1. **Create `apm-server/src/handlers/` directory** with two new files:
   - `handlers/mod.rs` — declares `pub mod tickets`
   - `handlers/tickets.rs` — start empty

2. **Add `mod handlers;` to `main.rs`** near the top, alongside the existing module declarations.

3. **Move handler-private helpers to `handlers/tickets.rs`** — cut from main.rs, paste into tickets.rs. Only these four helpers remain to move (business-logic helpers were already moved by prerequisite 2973f8d1):
   - `extract_section`
   - `extract_frontmatter_raw`
   - `extract_history_raw`
   - `load_tickets`

4. **Move handler functions to `handlers/tickets.rs`** — cut from main.rs, paste into tickets.rs:
   - `list_tickets`
   - `get_ticket`
   - `transition_ticket`
   - `put_body`
   - `patch_ticket`
   - `batch_transition`
   - `batch_priority`
   - `create_ticket`

5. **Add imports to `handlers/tickets.rs`**. The module needs:
   - `use crate::{AppError, AppState};` — shared error type and app state
   - `use crate::models::*;` — all ticket DTOs moved by prerequisite a6bc1326
   - All `use apm_core::...` paths needed by the moved functions — including the paths for `compute_blocking_deps` and `compute_valid_transitions` established by prerequisite 2973f8d1
   - `use axum::{extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}, Json};`
   - `use std::collections::{HashMap, HashSet};`
   - `use tokio::task::spawn_blocking;`
   - `use anyhow::{anyhow, Context};` as needed

6. **Update `build_app()` in `main.rs`** to reference handlers by qualified path:
   - `.route("/api/tickets", get(handlers::tickets::list_tickets).post(handlers::tickets::create_ticket))`
   - etc. for all ticket routes
   - Alternatively, add `use crate::handlers::tickets::*;` at module level in main.rs.

7. **Remove now-unused imports from `main.rs`** — any `use apm_core::...` or `use axum::extract::...` items that were only needed by the moved code, to avoid dead-code warnings.

8. **Compile and fix** — run `cargo build -p apm-server`; resolve any visibility or import errors. Common issues:
   - `AppError` / `AppState` in tickets.rs: import via `use crate::{AppError, AppState};`
   - Private helpers crossing the module boundary: make them `pub(crate)` as needed
   - Missing `crate::models` re-exports: ensure models.rs has `pub use` for all DTOs the handlers need

9. **Run tests** — `cargo test -p apm-server` to confirm nothing is broken.

**Constraints:**
- Do not rename any function or struct — only move them
- Do not change any function signatures or route paths
- `AppError` and `AppState` stay in main.rs
- DTOs live in `crate::models` (established by prerequisite a6bc1326) — do not re-define them in tickets.rs
- Business-logic functions (`compute_blocking_deps`, `compute_valid_transitions`) live in `apm_core` (established by prerequisite 2973f8d1) — do not re-define them in tickets.rs
- Line numbers in main.rs will have shifted after the prerequisite refactors; verify against the actual file before cutting

### Open questions


### Amendment requests

- [x] Remove all 14 ticket DTOs from scope (TransitionOption, TicketResponse, TicketsEnvelope, BlockingDep, TicketDetailResponse, TransitionRequest, BatchTransitionRequest, BatchPriorityRequest, PutBodyRequest, PatchTicketRequest, CreateTicketRequest, ListTicketsQuery, BatchFailure, BatchResult). Prerequisite a6bc1326 already moves these to `models.rs` — they will not be in `main.rs` when this ticket runs. The handlers should import them from `crate::models`.
- [x] Remove `compute_blocking_deps` and `compute_valid_transitions` from scope. Prerequisite 2973f8d1 already moves these to `apm_core`. The handlers should call them from `apm_core` directly.
- [x] Update acceptance criteria to only list: the 8 handler functions (list_tickets, get_ticket, transition_ticket, put_body, patch_ticket, create_ticket, batch_transition, batch_priority) and the handler-private helpers (extract_section, extract_frontmatter_raw, extract_history_raw, load_tickets).
- [x] Update the approach section to reflect that DTOs are imported from `crate::models::*` and business logic from `apm_core`, not moved into handlers/tickets.rs.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:45Z | groomed | in_design | philippepascal |
| 2026-04-12T09:48Z | in_design | specd | claude-0412-0945-5e00 |
| 2026-04-12T10:11Z | specd | ammend | apm |
| 2026-04-12T10:14Z | ammend | in_design | philippepascal |
| 2026-04-12T10:16Z | in_design | specd | claude-0412-1014-bbe0 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T11:22Z | ready | in_progress | philippepascal |
| 2026-04-12T11:30Z | in_progress | implemented | claude-0412-1122-9e28 |
| 2026-04-12T17:11Z | implemented | closed | philippepascal |
