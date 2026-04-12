+++
id = "9698c4c6"
title = "Extract clean and sync handlers from main.rs"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9698c4c6-extract-clean-and-sync-handlers-from-mai"
created_at = "2026-04-12T09:03:22.310905Z"
updated_at = "2026-04-12T09:57:10.746599Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["1ace7d42"]
+++

## Spec

### Problem

`apm-server/src/main.rs` is a ~4 000-line file that mixes HTTP handler logic, data structures, server bootstrap, and worker-queue code. Two maintenance-related handlers are currently defined inline in `main.rs`:

- `sync_handler` (lines 489–529, 41 lines): fetches git refs, syncs local ticket branches, applies sync rules to close tickets, and returns branch/closed counts.
- `CleanRequest` struct + `clean_handler` (lines 531–757, 227 lines combined): the single largest handler in the file. It mixes parameter parsing, blocking worktree candidate detection via `apm_core::clean`, dry-run response building, remote branch cleanup, and epic branch cleanup with TOML file manipulation.

The desired state: both handlers and their supporting struct live in `apm-server/src/handlers/maintenance.rs`, and `main.rs` retains only route registrations that reference the moved symbols by path. This mirrors the extraction pattern established for ticket and epic handlers (tickets 7bb8eacb and 1ace7d42) and reduces `main.rs` by ~270 lines.

This ticket depends on 1ace7d42 (epic handler extraction) being merged first. By that point `handlers/mod.rs` and `handlers/epics.rs` already exist, so this ticket only needs to add `pub mod maintenance;` to `handlers/mod.rs` and create the new file.

### Acceptance criteria

- [ ] `apm-server/src/handlers/maintenance.rs` exists and contains `sync_handler`
- [ ] `apm-server/src/handlers/maintenance.rs` exists and contains `clean_handler`
- [ ] `apm-server/src/handlers/maintenance.rs` exists and contains the `CleanRequest` struct
- [ ] `handlers/mod.rs` declares `pub mod maintenance;`
- [ ] `main.rs` references both handlers via `handlers::maintenance::` (directly or via a use import)
- [ ] `main.rs` no longer directly defines `sync_handler`, `clean_handler`, or `CleanRequest` (grep for each yields zero results in main.rs)
- [ ] `cargo build -p apm-server` succeeds with no compiler errors or warnings
- [ ] `cargo test -p apm-server` passes with all existing tests green (including `sync_in_memory_returns_not_implemented`)
- [ ] The HTTP routes registered in `build_app()` are unchanged — same verbs (`POST /api/sync`, `POST /api/clean`), same handler function bindings
- [ ] `AppError` and `AppState` remain defined in `main.rs`; `handlers/maintenance.rs` imports them from `crate`

### Out of scope

- Extracting non-maintenance handlers (auth, agents, workers, tickets, epics) — covered by sibling tickets 7bb8eacb and 1ace7d42
- Renaming any function, struct, or route path
- Changing any handler logic or behaviour
- Moving `AppError` or `AppState` out of `main.rs`
- Refactoring the epic cleanup block inside `clean_handler` to reuse `apm_core` helpers — that requires shared logic to exist in `apm_core` first and belongs to a separate ticket
- Adding new endpoints, response fields, or request parameters
- Writing tests that do not already exist

### Approach

This ticket runs after 1ace7d42 (epic handler extraction) is merged into the epic branch. By that point `apm-server/src/handlers/mod.rs`, `handlers/tickets.rs`, and `handlers/epics.rs` already exist. The following steps extend that structure.

**Assumed state from prior tickets:**
- `handlers/mod.rs` exists with at least `pub mod tickets;` and `pub mod epics;`
- `main.rs` declares `mod handlers;` and routes ticket/epic handlers via `handlers::`
- `sync_handler` is at lines ~489–529, `CleanRequest` at ~531–540, `clean_handler` at ~542–757

---

1. **Create `handlers/maintenance.rs`** (new file, initially empty).

2. **Add `pub mod maintenance;` to `handlers/mod.rs`** alongside existing `pub mod` lines.

3. **Move `CleanRequest` to `handlers/maintenance.rs`** — cut from `main.rs` with its `#[derive(serde::Deserialize, Default)]` attribute, paste into `maintenance.rs` as `pub struct CleanRequest { ... }`.

4. **Move `sync_handler` to `handlers/maintenance.rs`** — cut from `main.rs`, paste as `pub async fn sync_handler(...)`.

5. **Move `clean_handler` to `handlers/maintenance.rs`** — cut from `main.rs`, paste as `pub async fn clean_handler(...)`. The function body is unchanged; no refactoring of the epic cleanup block.

6. **Move the test to `handlers/maintenance.rs`** — cut `sync_in_memory_returns_not_implemented` from the `#[cfg(test)]` block in `main.rs` and place it in a `#[cfg(test)] mod tests { ... }` block inside `maintenance.rs`. The test helpers `build_app_with_tickets` and `test_tickets` remain in `main.rs`; import them via `crate::tests::build_app_with_tickets` and `crate::tests::test_tickets` (or whatever visibility the prior tickets established).

7. **Add imports to `handlers/maintenance.rs`**:
   ```rust
   use std::sync::Arc;
   use axum::{
       extract::State,
       http::StatusCode,
       response::{IntoResponse, Response},
       Json,
   };
   use crate::{AppError, AppState};
   ```
   The handler bodies reference `apm_core::*`, `serde_json`, `toml`, and `std::process::Command` — all are already in `Cargo.toml`; no new dependencies needed.

8. **Update route registrations in `main.rs`** — both occurrences (authenticated and unauthenticated app builders, lines ~1765–1766 and ~1839–1840) change from bare names to qualified paths:
   ```rust
   .route("/api/sync",  post(handlers::maintenance::sync_handler))
   .route("/api/clean", post(handlers::maintenance::clean_handler))
   ```

9. **Remove now-unused imports from `main.rs`** — any `use` items only needed by the moved handlers (e.g. `apm_core::clean::*`, `apm_core::sync::*` if nothing else uses them) should be removed to avoid dead-code warnings. Verify with `cargo build`.

10. **Compile and fix**:
    ```
    cargo build -p apm-server
    ```
    Likely issue: test helpers in `main.rs`'s `#[cfg(test)]` block may need to be `pub(crate)` for `maintenance.rs` tests to call them. Check visibility of `build_app_with_tickets`, `test_tickets`, `build_app`.

11. **Run tests**:
    ```
    cargo test -p apm-server
    ```

**Constraints:**
- Do not rename any function, struct, or route
- Do not change any function signatures or handler logic
- `AppError` and `AppState` stay in `main.rs`
- Line numbers are approximate against the pre-1ace7d42 state; verify against the actual file after the dependency is merged

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:53Z | groomed | in_design | philippepascal |