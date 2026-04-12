+++
id = "7bb8eacb"
title = "Extract ticket handlers from main.rs into handlers/tickets.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7bb8eacb-extract-ticket-handlers-from-main-rs-int"
created_at = "2026-04-12T09:03:09.497239Z"
updated_at = "2026-04-12T09:45:34.376132Z"
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
- [ ] `apm-server/src/handlers/tickets.rs` contains the helper functions: `extract_section`, `extract_frontmatter_raw`, `extract_history_raw`, `compute_blocking_deps`, `compute_valid_transitions`, `load_tickets`
- [ ] `apm-server/src/handlers/tickets.rs` contains all ticket-scoped request/response structs: `TransitionOption`, `TicketResponse`, `TicketsEnvelope`, `BlockingDep`, `TicketDetailResponse`, `TransitionRequest`, `BatchTransitionRequest`, `BatchPriorityRequest`, `PutBodyRequest`, `PatchTicketRequest`, `CreateTicketRequest`, `ListTicketsQuery`, `BatchFailure`, `BatchResult`
- [ ] `apm-server/src/handlers/mod.rs` exists and declares `pub mod tickets`
- [ ] `main.rs` imports handlers via `mod handlers` and references ticket handlers from `handlers::tickets`
- [ ] `main.rs` no longer directly defines any of the moved functions or structs (grep for their definition sites yields zero results in main.rs)
- [ ] `AppError` and `AppState` remain defined in `main.rs`; `handlers/tickets.rs` imports them from `super` or `crate`
- [ ] The HTTP routes registered in `build_app()` are unchanged — same verbs, same paths, same handler function bindings

### Out of scope

- Extracting non-ticket handlers (auth, agents, workers, login) — those are separate refactor tickets\n- Renaming any handler function, struct, or route path\n- Changing any handler's logic or behavior\n- Moving AppError or AppState out of main.rs\n- Adding new ticket endpoints or fields\n- Extracting spec-section handlers (get_ticket_spec, update_ticket_spec) if they were already moved by a prerequisite ticket — verify first and skip if already gone\n- Writing tests that do not already exist

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
| 2026-04-12T09:45Z | groomed | in_design | philippepascal |