+++
id = "a6bc1326"
title = "Extract request/response DTOs from main.rs into models.rs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a6bc1326-extract-request-response-dtos-from-main-"
created_at = "2026-04-12T09:02:56.242957Z"
updated_at = "2026-04-12T09:02:56.242957Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
+++

## Spec

### Problem

`apm-server/src/main.rs` (4,176 lines) defines 30+ request/response structs inline alongside handler logic. Examples include `TicketPatch`, `BatchUpdateRequest`, `ListTicketsQuery`, `EpicResponse`, `QueueResponse`, `LoginRequest`, `OtpVerifyRequest`, `WebAuthnRegisterStart`, and many more.

These DTOs are scattered throughout the file, interleaved with handler functions, making it hard to find or reuse them. They should be extracted into a dedicated `models.rs` (or `models/requests.rs` + `models/responses.rs`) module.

This is foundational work — subsequent tickets that extract handlers from main.rs will need the DTOs to already be in a shared location so multiple handler modules can import them.

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
| 2026-04-12T09:02Z | — | new | philippepascal |