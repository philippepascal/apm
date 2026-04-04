+++
id = "42f4b3ba"
title = "Add owner field to ticket frontmatter"
state = "ready"
priority = 0
effort = 4
risk = 2
author = "apm"
branch = "ticket/42f4b3ba-add-owner-field-to-ticket-frontmatter"
created_at = "2026-04-04T06:28:01.284791Z"
updated_at = "2026-04-04T16:58:24.303128Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

The ticket frontmatter has `author` (who created it) and `supervisor` (who reviews it) but no field to track who is currently working on it. The UI has an "agent" filter dropdown that renders but does nothing because there is no corresponding field in the Frontmatter struct or API response. Without an ownership field, there is no way to answer "which tickets is Alice currently responsible for?" — you can only see who created them.

### Acceptance criteria

- [ ] `Frontmatter` has an `owner` field that round-trips through TOML parse/serialize
- [ ] `apm set <id> owner <name>` sets the `owner` field
- [ ] `apm set <id> owner -` clears the `owner` field
- [ ] Unit tests cover the above three behaviours

### Out of scope

- Clearing `owner` automatically when a ticket leaves an active state — ownership tracking is intentionally sticky
- Back-filling `owner` on existing tickets in git history
- Enforcing that only one agent can own a ticket at a time
- Setting `owner` on `apm start` or `apm state in_design` transitions — covered by ticket ffaad988
- CLI filtering (`apm list --owner`) — covered by ticket b5b9b728
- API filtering (`GET /api/tickets?owner=`) and returning `owner` in API responses — covered by ticket 2b7c4c97
- UI filter wiring — covered by ticket 8f7dc4a3
- `apm take` / `apm assign` ownership handoff — covered by ticket 01dbdaad

### Approach

**apm-core/src/ticket.rs**

1. Add field to `Frontmatter` (after `supervisor`):
   `pub owner: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`.
2. Update `set_field`: add arm for `"owner"` — set to `None` when value is `"-"`, otherwise `Some(value.to_string())`.

**Tests** (inline in apm-core/src/ticket.rs or apm-core/tests/)

- TOML round-trip: parse frontmatter containing `owner = "alice"`, verify `fm.owner == Some("alice")`, serialize, verify field present in output.
- `set_field("owner", "alice")` → `fm.owner == Some("alice")`
- `set_field("owner", "-")` → `fm.owner == None`
- Frontmatter with no `owner` field deserializes without error (`fm.owner == None`).

**Order**

1. Add `owner` field to `Frontmatter`
2. Add `set_field` arm
3. Add tests
4. `cargo test --workspace` passes

### Open questions


### Amendment requests

- [x] Rename the field from `agent` to `owner` everywhere (Frontmatter, serde attribute, set_field arm, tests)
- [x] Strip acceptance criteria to ONLY: (1) `Frontmatter` has an `owner` field that round-trips through TOML parse/serialize, (2) `apm set <id> owner <name>` sets the field, (3) `apm set <id> owner -` clears it, (4) unit tests for the above
- [x] Remove from acceptance criteria: `apm start` setting owner (covered by ffaad988), `apm state in_design` setting owner (covered by ffaad988), `apm take` (being removed in 01dbdaad), `apm list --agent` (covered by b5b9b728), `GET /api/tickets?agent=` (covered by 2b7c4c97), UI filter wiring (covered by 8f7dc4a3)
- [x] Remove from approach: start.rs changes, state.rs changes, list.rs/main.rs CLI flag, server query param — all belong to dependent tickets
- [x] Update out-of-scope to mention that setting owner on transitions, CLI filtering, API filtering, and UI wiring are handled by dependent tickets

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:42Z | groomed | in_design | philippepascal |
| 2026-04-04T06:45Z | in_design | specd | claude-0404-0642-spec1 |
| 2026-04-04T07:14Z | specd | ammend | apm |
| 2026-04-04T07:45Z | ammend | in_design | philippepascal |
| 2026-04-04T07:47Z | in_design | specd | claude-0404-0800-spec2 |
| 2026-04-04T15:33Z | specd | ready | apm |
| 2026-04-04T16:09Z | ready | in_progress | philippepascal |
| 2026-04-04T16:58Z | in_progress | ready | apm |
