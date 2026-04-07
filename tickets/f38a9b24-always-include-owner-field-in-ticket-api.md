+++
id = "f38a9b24"
title = "Always include owner field in ticket API responses"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f38a9b24-always-include-owner-field-in-ticket-api"
created_at = "2026-04-06T20:57:23.971981Z"
updated_at = "2026-04-07T00:24:36.706423Z"
+++

## Spec

### Problem

The GET /api/tickets and GET /api/tickets/:id endpoints omit the owner field entirely when it is None. This forces every client to distinguish between 'field absent' and 'field is null', which is error-prone and inconsistent with how other optional fields (like author) are handled. The owner field should always be present in API responses — set to the username string when assigned, or null when unassigned. This applies to both the list and detail endpoints.

### Acceptance criteria

- [x] GET /api/tickets includes `"owner": null` for a ticket whose owner is unset
- [x] GET /api/tickets includes `"owner": "<username>"` for a ticket whose owner is set
- [x] GET /api/tickets/:id includes `"owner": null` for a ticket whose owner is unset
- [x] GET /api/tickets/:id includes `"owner": "<username>"` for a ticket whose owner is set
- [x] Existing `list_tickets_owner_field_absent` test asserts `arr[0]["owner"].is_null()` rather than accepting absence
- [x] New test: `get_ticket_owner_field_absent` asserts `json["owner"].is_null()`
- [x] New test: `get_ticket_owner_field_present` asserts `json["owner"] == "alice"`
- [x] `Ticket::serialize()` (TOML) is unaffected — absent owner still omits the key

### Out of scope

- Other optional fields (supervisor, branch, epic, etc.) — only owner
- Changing author's "unassigned" normalization
- Any client-side changes

### Approach

All changes in `apm-server/src/main.rs` only; `apm-core/` untouched.

Root cause: `Frontmatter.owner` has `#[serde(skip_serializing_if = "Option::is_none")]` which correctly protects TOML writes (TOML has no null). The fix must be at the API layer.

1. Add `owner: Option<String>` (without `skip_serializing_if`) to `TicketResponse` and `TicketDetailResponse`. This field serializes as `null` when `None`.
2. In `list_tickets` (~line 608): extract owner from frontmatter before serializing. Pass `owner` to `TicketResponse`.
3. In `get_ticket` (~line 652): extract owner from frontmatter before serializing. Pass `owner` to `TicketDetailResponse`.
4. Tighten `list_tickets_owner_field_absent` test: `assert!(arr[0]["owner"].is_null())`.
5. Add `get_ticket_owner_field_absent` and `get_ticket_owner_field_present` tests.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-06T20:57Z | — | new | philippepascal |
| 2026-04-06T21:22Z | new | groomed | apm |
| 2026-04-06T21:42Z | groomed | in_design | philippepascal |
| 2026-04-06T23:13Z | in_design | specd | claude-0406-1735-b2e1 |
| 2026-04-07T00:15Z | specd | ready | apm |
| 2026-04-07T00:24Z | ready | in_progress | philippepascal |
