+++
id = "1ace7d42"
title = "Extract epic handlers from main.rs into handlers/epics.rs"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1ace7d42-extract-epic-handlers-from-main-rs-into-"
created_at = "2026-04-12T09:03:14.832182Z"
updated_at = "2026-04-12T10:16:49.449462Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["7bb8eacb"]
+++

## Spec

### Problem

`apm-server/src/main.rs` is a ~4 000-line file that mixes HTTP handler logic, data structures, server bootstrap, and worker-queue code. Epic-related handler code — roughly 220 lines of production code plus ~185 lines of tests — is defined inline in main.rs alongside unrelated concerns.

The desired state mirrors the pattern established by the sibling ticket for ticket handlers (7bb8eacb): all epic HTTP handler code lives in `apm-server/src/handlers/epics.rs`, and `main.rs` retains only route registrations that reference the moved functions by path. After both extractions main.rs shrinks by ~700 lines total and is navigable by concern.

Note on `parse_epic_branch`: this function duplicates the slug-to-title logic in `apm/src/cmd/epic.rs`. Moving it to `handlers/epics.rs` as-is is correct for this ticket. Replacing it with a shared `apm_core::epic` helper is explicitly out of scope and belongs to a separate refactor.

### Acceptance criteria

- [ ] `apm-server/src/handlers/epics.rs` exists and contains all three HTTP handler functions: `list_epics`, `create_epic`, `get_epic`
- [ ] `apm-server/src/handlers/epics.rs` contains the helper functions: `parse_epic_branch`, `derive_epic_state`, `build_epic_summary`, `find_epic_branch`
- [ ] `apm-server/src/handlers/epics.rs` contains the request/response structs: `EpicSummary`, `EpicDetailResponse`, `CreateEpicRequest`
- [ ] `apm-server/src/handlers/mod.rs` declares `pub mod epics`
- [ ] `main.rs` references epic handlers via `handlers::epics::` (directly or via a use import)
- [ ] `main.rs` no longer directly defines any of the moved functions or structs (grep for `fn list_epics`, `fn create_epic`, `fn get_epic`, `fn parse_epic_branch`, `fn derive_epic_state`, `fn build_epic_summary`, `fn find_epic_branch`, `struct EpicSummary`, `struct EpicDetailResponse`, `struct CreateEpicRequest` yields zero results in main.rs)
- [ ] `cargo build -p apm-server` succeeds with no compiler errors or warnings
- [ ] `cargo test -p apm-server` passes with all existing tests green
- [ ] The HTTP routes registered in `build_app()` are unchanged — same verbs (`GET /api/epics`, `POST /api/epics`, `GET /api/epics/:id`), same handler function bindings
- [ ] `AppError` and `AppState` remain defined in `main.rs`; `handlers/epics.rs` imports them from `crate`

### Out of scope

- Extracting non-epic handlers (auth, agents, workers, tickets) — tickets are covered by 7bb8eacb
- Renaming any function, struct, or route path
- Changing any handler's logic or behaviour
- Moving `AppError` or `AppState` out of `main.rs`
- Replacing `parse_epic_branch` / `derive_epic_state` with shared `apm_core::epic` helpers — that requires a separate refactor once those helpers exist in apm_core
- Adding new epic endpoints or response fields
- Writing tests that do not already exist

### Approach

This ticket runs after 7bb8eacb (ticket-handler extraction) is merged. By that point `apm-server/src/handlers/mod.rs` and `apm-server/src/handlers/tickets.rs` already exist. The following steps extend that structure.

**Prerequisite state assumed from 7bb8eacb:**
- `handlers/mod.rs` exists with at least `pub mod tickets;`
- `handlers/tickets.rs` contains `TicketResponse`, `extract_section`, and `load_tickets` as `pub(crate)` items
- `main.rs` declares `mod handlers;` and calls ticket handlers via `handlers::tickets::`
- `find_epic_branch` is still in `main.rs` (it was left there as a `crate`-visible helper for `create_ticket`, which lives in `handlers/tickets.rs` and calls it as `crate::find_epic_branch`)

---

1. **Create `handlers/epics.rs`** (new file, initially empty).

2. **Add `pub mod epics;` to `handlers/mod.rs`** alongside the existing `pub mod tickets;` line.

3. **Move structs to `handlers/epics.rs`** — cut from `main.rs` with all `#[derive]` / `#[serde]` attributes, paste into `epics.rs`:
   - `EpicSummary` (lines ~165–172)
   - `EpicDetailResponse` (lines ~174–179)
   - `CreateEpicRequest` (lines ~181–184)

4. **Move helper functions to `handlers/epics.rs`** — cut from `main.rs`, paste into `epics.rs`:
   - `find_epic_branch` (lines ~161–163) — make it `pub(crate)` so `handlers/tickets.rs` can continue to call it (see step 8)
   - `parse_epic_branch` (lines ~186–203)
   - `derive_epic_state` (lines ~205–249)
   - `build_epic_summary` (lines ~251–273)

5. **Move handler functions to `handlers/epics.rs`**:
   - `list_epics` (lines ~275–295)
   - `create_epic` (lines ~297–325)
   - `get_epic` (lines ~327–381)

6. **Move epic tests to `handlers/epics.rs`** — cut the following test functions from the `#[cfg(test)]` block in `main.rs` and place them in a `#[cfg(test)] mod tests { ... }` block inside `epics.rs`. Tests to move:
   - `list_epics_in_memory_returns_501` (line ~2999)
   - `create_epic_missing_title_returns_400` (line ~3014)
   - `create_epic_empty_title_returns_400` (line ~3034)
   - `create_epic_in_memory_returns_501` (line ~3054)
   - `get_epic_in_memory_returns_501` (line ~3071)
   - `get_epic_not_found_returns_404` (line ~3086)
   - `list_epics_empty_returns_empty_array` (line ~3104)
   - `create_epic_round_trip` (line ~3125)

   The test helpers `build_app`, `build_app_with_tickets`, `test_tickets`, `git_setup` remain in `main.rs`; import them via `crate::tests::*` or equivalent.

7. **Add imports to `handlers/epics.rs`**:
   ```rust
   use std::collections::HashMap;
   use std::sync::Arc;
   use axum::{
       extract::{Path, State},
       http::StatusCode,
       response::{IntoResponse, Response},
       Json,
   };
   use crate::{AppError, AppState};
   use crate::handlers::tickets::{extract_section, load_tickets, TicketResponse};
   ```

8. **Update `handlers/tickets.rs`** — `create_ticket` currently calls `crate::find_epic_branch`. After step 4 above moves that function, change the call site to `crate::handlers::epics::find_epic_branch` (or add `use crate::handlers::epics::find_epic_branch;` at the top of tickets.rs).

9. **Update route registrations in `main.rs`** — replace bare handler names with fully-qualified paths (or add a use import). Both occurrences at lines ~1782–1783 and ~1845–1846 change from:
   ```rust
   .route("/api/epics", get(list_epics).post(create_epic))
   .route("/api/epics/:id", get(get_epic))
   ```
   to:
   ```rust
   .route("/api/epics", get(handlers::epics::list_epics).post(handlers::epics::create_epic))
   .route("/api/epics/:id", get(handlers::epics::get_epic))
   ```

10. **Remove now-unused imports from `main.rs`** — any `use` items that were only needed by the moved code (e.g. `apm_core::epic::epic_branches`, `apm_core::epic::create_epic_branch`) should be removed to avoid dead-code warnings. Keep any that are still used by remaining code.

11. **Compile and fix**:
    ```
    cargo build -p apm-server
    ```
    Common issues to watch for:
    - `TicketResponse`, `extract_section`, `load_tickets` visibility — they must be `pub(crate)` in `handlers/tickets.rs`
    - `EpicDetailResponse` references `TicketResponse` from a sibling module — the import in step 7 covers this
    - Test helpers in `main.rs` tests block may need to be `pub(crate)` for `handlers/epics.rs` tests to access them

12. **Run tests**:
    ```
    cargo test -p apm-server
    ```

**Constraints:**
- Do not rename any function, struct, or route path
- Do not change any function signatures or handler logic
- `AppError` and `AppState` stay in `main.rs`
- Line numbers are approximate against the pre-7bb8eacb state; verify against the actual file after the dependency is merged

### Open questions


### Amendment requests

- [ ] Remove EpicSummary, EpicDetailResponse, and CreateEpicRequest from scope. Prerequisite a6bc1326 already moves these to `models.rs` — they will not be in `main.rs` when this ticket runs. The handlers should import them from `crate::models`.
- [ ] Update acceptance criteria: remove the line about handlers/epics.rs containing request/response structs. The file should only contain handler functions and helper functions.
- [ ] Update the approach: remove step 3 (move structs) entirely, and update imports in step 7 to include `use crate::models::{EpicSummary, EpicDetailResponse, CreateEpicRequest};` instead of defining them locally.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:49Z | groomed | in_design | philippepascal |
| 2026-04-12T09:53Z | in_design | specd | claude-0412-0949-cb30 |
| 2026-04-12T10:11Z | specd | ammend | apm |
| 2026-04-12T10:16Z | ammend | in_design | philippepascal |
