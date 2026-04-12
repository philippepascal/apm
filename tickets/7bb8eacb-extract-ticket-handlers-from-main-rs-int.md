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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

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