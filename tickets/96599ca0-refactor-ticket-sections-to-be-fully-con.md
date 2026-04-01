+++
id = "96599ca0"
title = "Refactor ticket sections to be fully config-driven, removing hardcoded TicketDocument fields"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "44317"
branch = "ticket/96599ca0-refactor-ticket-sections-to-be-fully-con"
created_at = "2026-04-01T22:27:39.127351Z"
updated_at = "2026-04-01T22:35:17.030477Z"
+++

## Spec

### Problem

`TicketDocument` in `apm-core/src/ticket.rs` (~line 503) hardcodes the ticket body as six typed Rust fields (`problem`, `acceptance_criteria`, `out_of_scope`, `approach`, `open_questions`, `amendment_requests`). `spec.rs` has matching hardcoded arms in `get_section`, `set_section`, and `is_doc_field`. Section order in serialization is also hardcoded.

The config already defines sections properly via `[[ticket.sections]]` in `.apm/config.toml` (name, type, required, placeholder), but this config is only used for skeleton generation and CLI validation — not at the model layer.

The consequence: adding any new section (e.g. a delegator-facing Context field) requires Rust code changes in `ticket.rs` and `spec.rs` instead of a config entry. Worse, sections not in `TicketDocument` get silently dropped on the next round-trip through `serialize`.

The fix is to replace `TicketDocument`'s typed fields with a config-driven ordered map. The server `CreateTicketRequest` in `apm-server/src/main.rs` (~line 91) also hardcodes the four main section fields — breaking that API is acceptable; the only client is `apm-ui` which must be fixed in the same PR.

### Acceptance criteria

- [ ] Adding a new entry to `[[ticket.sections]]` in `.apm/config.toml` makes that section appear in newly created ticket skeletons without any Rust code changes
- [ ] `apm spec <id> --section <name>` works for any section defined in config, not only the six currently hardcoded ones
- [ ] A ticket containing a section whose name is not in config is preserved unchanged on a parse → serialize round-trip (no silent drops)
- [ ] The "Code review" section (present in config but absent from `TicketDocument`) survives a parse-serialize round-trip on an existing ticket file
- [ ] `TicketDocument` no longer declares individual typed Rust fields (`problem`, `acceptance_criteria`, etc.) — sections are stored in an ordered map
- [ ] `get_section` and `set_section` in `spec.rs` contain no hardcoded section-name match arms
- [ ] `is_doc_field` in `spec.rs` is driven by the config section list, not a hardcoded string literal list
- [ ] `CreateTicketRequest` in `apm-server/src/main.rs` no longer has individual named section fields; it accepts a generic sections map
- [ ] `apm-ui` `NewTicketModal` sends form data using the new generic sections map shape
- [ ] `cargo test --workspace` passes with no new failures after the refactor

### Out of scope

- Making the `apm-ui` form dynamically fetch section definitions from a config API endpoint (form fields remain hardcoded in the UI, only the payload shape changes)
- Adding new sections to the default `.apm/config.toml` (this ticket only makes new sections work once added; it does not add any)
- Changing the `SectionType` enum values or config parsing logic in `config.rs`
- Migrating existing ticket files on disk — old files continue to round-trip correctly
- Changes to the `apm check` command beyond what is required to compile (if `apm check` accesses typed fields it will be updated to parse the raw string, but no behaviour changes)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:27Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:28Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:35Z | groomed | in_design | philippepascal |