+++
id = "a6bc1326"
title = "Extract request/response DTOs from main.rs into models.rs"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a6bc1326-extract-request-response-dtos-from-main-"
created_at = "2026-04-12T09:02:56.242957Z"
updated_at = "2026-04-12T10:59:28.123760Z"
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

**File to create:** `apm-server/src/models.rs`
**File to modify:** `apm-server/src/main.rs`

### 1. Create `models.rs`

Create `apm-server/src/models.rs`. Group structs by domain with comments for readability, in the order they appear in `main.rs`:

```
// Ticket DTOs (lines 58–165 in main.rs)
TransitionOption, TicketResponse, TicketsEnvelope, BlockingDep,
TicketDetailResponse, TransitionRequest, BatchTransitionRequest,
BatchPriorityRequest, BatchFailure, BatchResult, PutBodyRequest,
PatchTicketRequest, CreateTicketRequest

// Epic DTOs (lines 166–192)
EpicSummary, EpicDetailResponse, CreateEpicRequest

// Misc handler DTOs (scattered — lines 532, 760)
CleanRequest, ListTicketsQuery

// Auth/WebAuthn DTOs (lines 1460–1653)
RegisterChallengeRequest, RegisterChallengeResponse,
RegisterCompleteRequest, LoginChallengeRequest,
LoginChallengeResponse, LoginCompleteRequest
```

Copy each struct verbatim from `main.rs`, adding `pub` to the struct keyword if not already present. Preserve all `#[derive(...)]`, `#[serde(...)]`, and doc comments exactly. Add any `use`/`serde` imports that the structs require at the top of `models.rs` (e.g., `use serde::{Serialize, Deserialize};` if derives use the short form).

### 2. Update `main.rs`

- Add `mod models;` to the existing `mod` block (lines 15–23, after the last existing `mod` line).
- Add `use models::*;` immediately after the `mod` declarations so all handlers continue to resolve the DTO types without any other changes.
- Delete the 24 struct definitions from `main.rs`. The structs are at lines: 58–165 (ticket DTOs), 166–192 (epic DTOs), 532 (CleanRequest), 760 (ListTicketsQuery), 1460–1653 (auth DTOs). Delete each block in place; do not touch surrounding handler code.

### 3. Verify

Run `cargo build` and `cargo test` in `apm-server/`. Fix any compile errors that arise from missing imports in `models.rs` (e.g., types from `webauthn_rs`, `uuid`, etc. referenced inside the structs). No logic changes are needed — this is a pure mechanical extraction.

### Known constraints

- Some auth DTO fields reference types from `webauthn_rs` or `base64` crates. Ensure those `use` statements are present in `models.rs` or that the types are fully qualified.
- `serde` derives in `main.rs` use the path form `#[derive(serde::Serialize)]` — copy them as-is; no need to add a `use serde` import for derives in that form.
- The target branch for this ticket is `epic/1e706443-refactor-apm-server-code-organization`, so the PR should target that branch, not `main`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:34Z | groomed | in_design | philippepascal |
| 2026-04-12T09:36Z | in_design | specd | claude-0412-0934-4b38 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T10:59Z | ready | in_progress | philippepascal |
