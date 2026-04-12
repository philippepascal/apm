+++
id = "1ace7d42"
title = "Extract epic handlers from main.rs into handlers/epics.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1ace7d42-extract-epic-handlers-from-main-rs-into-"
created_at = "2026-04-12T09:03:14.832182Z"
updated_at = "2026-04-12T09:49:07.015419Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:49Z | groomed | in_design | philippepascal |