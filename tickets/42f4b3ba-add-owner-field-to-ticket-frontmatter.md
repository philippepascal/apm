+++
id = "42f4b3ba"
title = "Add owner field to ticket frontmatter"
state = "ammend"
priority = 0
effort = 4
risk = 2
author = "apm"
branch = "ticket/42f4b3ba-add-owner-field-to-ticket-frontmatter"
created_at = "2026-04-04T06:28:01.284791Z"
updated_at = "2026-04-04T07:14:52.803355Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

The ticket frontmatter has `author` (who created it) and `supervisor` (who reviews it) but no field to track who is currently working on it. The UI has an "agent" filter dropdown that renders but does nothing because there is no corresponding field in the Frontmatter struct or API response. Without an ownership field, there is no way to answer "which tickets is Alice currently responsible for?" — you can only see who created them.

### Acceptance criteria

- [ ] `Frontmatter` has an `agent` field that round-trips through TOML parse/serialize
- [ ] `apm start <id>` sets `agent` on the ticket frontmatter to the running agent's name
- [ ] `apm state <id> in_design` sets `agent` on the ticket frontmatter to the running agent's name
- [ ] `apm take <id>` sets `agent` to the new agent's name (replacing the previous value) and records the correct previous agent name in the history row instead of "unknown"
- [ ] `apm set <id> agent <name>` sets the `agent` field; `apm set <id> agent -` clears it
- [ ] `apm list --agent <name>` filters to tickets whose `agent` field matches (analogous to existing `--author` filter)
- [ ] `GET /api/tickets?agent=<name>` returns only tickets whose `agent` matches
- [ ] `GET /api/tickets` includes `agent` in each ticket object when set (null/absent when not set)
- [ ] The UI agent filter dropdown is populated from the `agent` values in the ticket list and correctly filters the swimlane view

### Out of scope

- Clearing `agent` automatically when a ticket leaves an active state (e.g. transitions to `specd`, `blocked`, or `closed`) — ownership tracking is intentionally sticky
- UI changes to the TicketCard or TicketDetail components (the UI already reads `ticket.agent` and renders it; once the API returns the field the UI will work)
- Back-filling `agent` on existing tickets in git history
- Enforcing that only one agent can own a ticket at a time

### Approach

**apm-core/src/ticket.rs**

1. Add field to Frontmatter (after supervisor):
   Add `pub agent: Option<String>` with `skip_serializing_if = "Option::is_none"`.
2. Update set_field: add arm for "agent" -- set to None when value is `-`, otherwise Some(value).
3. Update handoff: read `ticket.frontmatter.agent.clone().unwrap_or_else(|| "unknown".to_string())` as the from value, then set `ticket.frontmatter.agent = Some(new_agent.to_string())`.
4. Update list_filtered: add `agent_filter: Option<&str>` parameter and filter on `fm.agent`.

**apm-core/src/start.rs**

In `start::run`, after setting `t.frontmatter.state`, add:
`t.frontmatter.agent = Some(agent_name.to_string());`

**apm-core/src/state.rs**

In `transition`, after resolving `actor`, when the target state is "in_design" set:
`t.frontmatter.agent = Some(actor.clone());`

**apm/src/cmd/list.rs + apm/src/main.rs**

- Add `agent: Option<String>` parameter to `list::run`
- Add `--agent` clap flag in `apm/src/main.rs` and pass through to `list_filtered`

**apm-server/src/main.rs**

- Add `agent: Option<String>` to `ListTicketsQuery`
- After the existing `author` filter block, add an analogous block filtering by `params.agent`

**Tests**

- `list_filtered` with `agent_filter` in ticket.rs unit tests
- `handoff` test verifying `fm.agent` is updated and history uses the old value (not "unknown")
- `set_field` test for "agent" and "-"
- Server tests `list_tickets_agent_filter` and `list_tickets_agent_field_in_response`

**Order**

1. ticket.rs changes (Frontmatter + set_field + handoff + list_filtered)
2. start.rs agent assignment on start
3. state.rs agent assignment on in_design
4. CLI list flag
5. Server query param
6. cargo test --workspace passes

### Open questions


### Amendment requests

- [ ] Rename the field from `agent` to `owner` everywhere (Frontmatter, serde attribute, set_field arm, tests)
- [ ] Strip acceptance criteria to ONLY: (1) `Frontmatter` has an `owner` field that round-trips through TOML parse/serialize, (2) `apm set <id> owner <name>` sets the field, (3) `apm set <id> owner -` clears it, (4) unit tests for the above
- [ ] Remove from acceptance criteria: `apm start` setting owner (covered by ffaad988), `apm state in_design` setting owner (covered by ffaad988), `apm take` (being removed in 01dbdaad), `apm list --agent` (covered by b5b9b728), `GET /api/tickets?agent=` (covered by 2b7c4c97), UI filter wiring (covered by 8f7dc4a3)
- [ ] Remove from approach: start.rs changes, state.rs changes, list.rs/main.rs CLI flag, server query param — all belong to dependent tickets
- [ ] Update out-of-scope to mention that setting owner on transitions, CLI filtering, API filtering, and UI wiring are handled by dependent tickets

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:42Z | groomed | in_design | philippepascal |
| 2026-04-04T06:45Z | in_design | specd | claude-0404-0642-spec1 |
| 2026-04-04T07:14Z | specd | ammend | apm |
