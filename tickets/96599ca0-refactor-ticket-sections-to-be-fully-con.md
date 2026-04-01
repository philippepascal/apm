+++
id = "96599ca0"
title = "Refactor ticket sections to be fully config-driven, removing hardcoded TicketDocument fields"
state = "groomed"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
branch = "ticket/96599ca0-refactor-ticket-sections-to-be-fully-con"
created_at = "2026-04-01T22:27:39.127351Z"
updated_at = "2026-04-01T22:28:33.814950Z"
+++

## Spec

### Problem

`TicketDocument` in `apm-core/src/ticket.rs` (~line 503) hardcodes the ticket body as six typed Rust fields (`problem`, `acceptance_criteria`, `out_of_scope`, `approach`, `open_questions`, `amendment_requests`). `spec.rs` has matching hardcoded arms in `get_section`, `set_section`, and `is_doc_field`. Section order in serialization is also hardcoded.

The config already defines sections properly via `[[ticket.sections]]` in `.apm/config.toml` (name, type, required, placeholder), but this config is only used for skeleton generation and CLI validation — not at the model layer.

The consequence: adding any new section (e.g. a delegator-facing Context field) requires Rust code changes in `ticket.rs` and `spec.rs` instead of a config entry. Worse, sections not in `TicketDocument` get silently dropped on the next round-trip through `serialize`.

The fix is to replace `TicketDocument`'s typed fields with a config-driven ordered map. The server `CreateTicketRequest` in `apm-server/src/main.rs` (~line 91) also hardcodes the four main section fields — breaking that API is acceptable; the only client is `apm-ui` which must be fixed in the same PR.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |