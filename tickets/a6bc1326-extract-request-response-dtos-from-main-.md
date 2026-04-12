+++
id = "a6bc1326"
title = "Extract request/response DTOs from main.rs into models.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a6bc1326-extract-request-response-dtos-from-main-"
created_at = "2026-04-12T09:02:56.242957Z"
updated_at = "2026-04-12T09:34:00.269907Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
+++

## Spec

### Problem

`apm-server/src/main.rs` (4,176 lines) defines 24 request/response structs inline, interleaved with handler logic and infrastructure code. These DTOs span multiple domains — tickets, epics, auth/WebAuthn — but are all colocated in a single file, making them hard to locate and impossible to import from future handler modules.

The desired state is a dedicated `models.rs` sibling module containing all 24 DTOs, with `main.rs` declaring the module and importing from it. No other source files currently reference these structs, so the extraction is self-contained.

This is foundational work. Subsequent tickets that split handlers out of `main.rs` into their own modules will need to `use crate::models::*` (or specific imports). If the DTOs remain in `main.rs` when those tickets land, handler modules will be unable to reference them without a circular dependency.

### Acceptance criteria

- [ ] `apm-server/src/models.rs` exists and contains all 24 DTOs: `TransitionOption`, `TicketResponse`, `TicketsEnvelope`, `BlockingDep`, `TicketDetailResponse`, `BatchFailure`, `BatchResult`, `EpicSummary`, `EpicDetailResponse`, `RegisterChallengeResponse`, `LoginChallengeResponse`, `TransitionRequest`, `BatchTransitionRequest`, `BatchPriorityRequest`, `PutBodyRequest`, `PatchTicketRequest`, `CreateTicketRequest`, `CreateEpicRequest`, `CleanRequest`, `ListTicketsQuery`, `RegisterChallengeRequest`, `RegisterCompleteRequest`, `LoginChallengeRequest`, `LoginCompleteRequest`
- [ ] Every struct in `models.rs` is declared `pub`
- [ ] All original `#[derive(...)]` attributes on each struct are preserved exactly
- [ ] `main.rs` declares `mod models;` in its module block
- [ ] `main.rs` imports the DTOs via `use models::*;` (or equivalent explicit imports) so all existing handler code compiles without changes
- [ ] None of the 24 DTO struct definitions remain in `main.rs`
- [ ] `AppState`, `TicketSource`, and `AppError` remain in `main.rs` (not moved)
- [ ] `cargo build` in `apm-server/` succeeds with no errors
- [ ] `cargo test` in `apm-server/` passes (no regressions)

### Out of scope

- Splitting into `models/requests.rs` + `models/responses.rs` subdirectory layout — a flat `models.rs` is sufficient for now
- Moving `AppState`, `TicketSource`, or `AppError` — those are infrastructure, not DTOs
- Extracting handler functions from `main.rs` — covered by subsequent tickets in this epic
- Adding, removing, or renaming any DTO fields or derives
- Adding new DTOs beyond those already in `main.rs`
- Re-exporting models from a crate-level `lib.rs` — `apm-server` is a binary crate

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:34Z | groomed | in_design | philippepascal |