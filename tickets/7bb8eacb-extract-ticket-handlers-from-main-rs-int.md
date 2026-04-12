+++
id = "7bb8eacb"
title = "Extract ticket handlers from main.rs into handlers/tickets.rs"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7bb8eacb-extract-ticket-handlers-from-main-rs-int"
created_at = "2026-04-12T09:03:09.497239Z"
updated_at = "2026-04-12T10:14:05.125080Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["a6bc1326", "2973f8d1"]
+++

## Spec

### Problem

main.rs in apm-server currently contains roughly 500 lines of ticket-related HTTP handler code — request/response structs, helper functions, and eight async handler functions — all mixed in with server bootstrap, auth routes, and worker-queue code. This makes the file very long (~4000+ lines) and hard to navigate.\n\nThe desired state is that all ticket CRUD logic lives in a dedicated handlers/tickets.rs module. After extraction, main.rs retains only route registration (referencing handlers by path) and the cross-cutting infrastructure types (AppState, AppError, server startup). This mirrors the existing pattern for other logical groupings in the codebase (agents.rs, auth.rs, workers.rs, etc.).\n\nThis ticket depends on two prerequisite refactors (a6bc1326 and 2973f8d1) that move DTOs and core business logic out of main.rs, so by the time this work runs, the ticket handler code is already relatively self-contained.

### Acceptance criteria

- [ ] `cargo build -p apm-server` succeeds after the extraction with no compiler errors or warnings
- [ ] `cargo test -p apm-server` passes (all existing tests continue to pass)
- [ ] `apm-server/src/handlers/tickets.rs` exists and contains all eight handler functions: `list_tickets`, `get_ticket`, `transition_ticket`, `put_body`, `patch_ticket`, `create_ticket`, `batch_transition`, `batch_priority`
- [ ] `apm-server/src/handlers/tickets.rs` contains the handler-private helper functions: `extract_section`, `extract_frontmatter_raw`, `extract_history_raw`, `load_tickets`
- [ ] `apm-server/src/handlers/mod.rs` exists and declares `pub mod tickets`
- [ ] `main.rs` imports handlers via `mod handlers` and references ticket handlers from `handlers::tickets`
- [ ] `main.rs` no longer directly defines any of the moved functions (grep for their definition sites yields zero results in main.rs)
- [ ] `AppError` and `AppState` remain defined in `main.rs`; `handlers/tickets.rs` imports them from `crate`
- [ ] `handlers/tickets.rs` imports ticket DTOs via `use crate::models::*` (not re-defining them)
- [ ] `handlers/tickets.rs` calls `compute_blocking_deps` and `compute_valid_transitions` from `apm_core` (not defining them locally)
- [ ] The HTTP routes registered in `build_app()` are unchanged — same verbs, same paths, same handler function bindings

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
   - `handlers/tickets.rs` — all moved code (start empty)

2. **Add `mod handlers;` to `main.rs`** near the top, alongside the existing module declarations.

3. **Move structs to `handlers/tickets.rs`** — cut from main.rs (with their `#[derive]` and `#[serde]` attributes), paste into tickets.rs. Items to move:
   - `TransitionOption` (lines ~57–63)
   - `TicketResponse` (lines ~65–74)
   - `TicketsEnvelope` (lines ~76–80)
   - `BlockingDep` (lines ~94–98)
   - `TicketDetailResponse` (lines ~100–109)
   - `TransitionRequest` (lines ~111–114)
   - `BatchTransitionRequest` (lines ~116–120)
   - `BatchPriorityRequest` (lines ~122–126)
   - `BatchFailure` (lines ~128–131)
   - `BatchResult` (lines ~134–138)
   - `PutBodyRequest` (lines ~140–143)
   - `PatchTicketRequest` (lines ~145–151)
   - `CreateTicketRequest` (lines ~153–159)
   - `ListTicketsQuery` (lines ~760–764)

4. **Move helper functions to `handlers/tickets.rs`** — cut from main.rs, paste into tickets.rs:
   - `extract_section` (lines ~82–91)
   - `extract_frontmatter_raw` (lines ~383–387)
   - `extract_history_raw` (lines ~389–394)
   - `compute_blocking_deps` (lines ~416–443)
   - `compute_valid_transitions` (lines ~445–469)
   - `load_tickets` (lines ~471–483)

5. **Move handler functions to `handlers/tickets.rs`** — cut from main.rs, paste into tickets.rs:
   - `list_tickets` (lines ~766–854)
   - `get_ticket` (lines ~856–901)
   - `transition_ticket` (lines ~903–973)
   - `put_body` (lines ~975–1078)
   - `patch_ticket` (lines ~1080–1180)
   - `batch_transition` (lines ~1182–1206)
   - `batch_priority` (lines ~1208–1273)
   - `create_ticket` (lines ~1275–1357)

6. **Add imports to `handlers/tickets.rs`**. The module needs:
   - `use crate::{AppError, AppState};` — for the shared error type and app state
   - All `use apm_core::...` statements currently used by the moved functions (ticket, state, config, git, epic modules)
   - `use axum::{extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}, Json};`
   - `use serde::{Deserialize, Serialize};`
   - `use std::collections::{HashMap, HashSet};`
   - `use tokio::task::spawn_blocking;`
   - `use anyhow::Context;` / `use anyhow::anyhow;` as needed

7. **Update `build_app()` in `main.rs`**. Replace bare handler names with `handlers::tickets::` prefixed names, e.g.:
   - `.route("/api/tickets", get(handlers::tickets::list_tickets).post(handlers::tickets::create_ticket))`
   - etc.
   Alternatively add `use crate::handlers::tickets::*;` at the top of the `build_app` function or at the module level.

8. **Remove now-unused imports from `main.rs`** — any `use apm_core::...` or `use axum::extract::...` items that were only needed by the moved code should be removed to avoid dead-code warnings.

9. **Compile and fix** — run `cargo build -p apm-server`; resolve any visibility, import, or type-reference errors. Common issues:
   - `AppError` referenced in tickets.rs: import via `use crate::AppError;`
   - `AppState` referenced in tickets.rs: import via `use crate::AppState;`
   - Private helpers referenced across modules: make them `pub(crate)` or `pub` as needed
   - Any `use` of `TicketSource` or other AppState-adjacent types: import from `crate`

10. **Run tests** — `cargo test -p apm-server` to confirm nothing is broken.

**Constraints:**
- Do not rename any function or struct — only move them
- Do not change any function signatures or route paths
- `AppError` and `AppState` stay in main.rs (they are used by non-ticket routes too)
- Line numbers above are approximate; verify against the actual file before cutting

### Open questions


### Amendment requests

- [ ] Remove all 14 ticket DTOs from scope (TransitionOption, TicketResponse, TicketsEnvelope, BlockingDep, TicketDetailResponse, TransitionRequest, BatchTransitionRequest, BatchPriorityRequest, PutBodyRequest, PatchTicketRequest, CreateTicketRequest, ListTicketsQuery, BatchFailure, BatchResult). Prerequisite a6bc1326 already moves these to `models.rs` — they will not be in `main.rs` when this ticket runs. The handlers should import them from `crate::models`.
- [ ] Remove `compute_blocking_deps` and `compute_valid_transitions` from scope. Prerequisite 2973f8d1 already moves these to `apm_core`. The handlers should call them from `apm_core` directly.
- [ ] Update acceptance criteria to only list: the 8 handler functions (list_tickets, get_ticket, transition_ticket, put_body, patch_ticket, create_ticket, batch_transition, batch_priority) and the handler-private helpers (extract_section, extract_frontmatter_raw, extract_history_raw, load_tickets).
- [ ] Update the approach section to reflect that DTOs are imported from `crate::models::*` and business logic from `apm_core`, not moved into handlers/tickets.rs.

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