+++
id = "9698c4c6"
title = "Extract clean and sync handlers from main.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9698c4c6-extract-clean-and-sync-handlers-from-mai"
created_at = "2026-04-12T09:03:22.310905Z"
updated_at = "2026-04-12T09:53:55.997471Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:53Z | groomed | in_design | philippepascal |