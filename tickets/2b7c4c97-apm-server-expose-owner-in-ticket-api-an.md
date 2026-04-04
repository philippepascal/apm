+++
id = "2b7c4c97"
title = "apm-server: expose owner in ticket API and add owner query param"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "apm"
branch = "ticket/2b7c4c97-apm-server-expose-owner-in-ticket-api-an"
created_at = "2026-04-04T06:28:16.243562Z"
updated_at = "2026-04-04T06:58:41.810540Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

The `/api/tickets` list endpoint does not expose the `agent` field (the person or AI agent currently assigned to work on a ticket). The `ListTicketsQuery` struct supports `?author=` filtering but has no `?agent=` query parameter.

Ticket #42f4b3ba adds `agent: Option<String>` to `Frontmatter`. Because `TicketResponse` flattens `Frontmatter` via `#[serde(flatten)]`, the field will appear in responses automatically once the dependency lands — but only when non-null (`skip_serializing_if = "Option::is_none"`). The missing piece is the server-side `?agent=` query param, which the UI's agent filter dropdown needs to perform filtered fetches rather than client-side filtering over the full ticket list.

### Acceptance criteria

- [ ] `GET /api/tickets` includes `agent` in each ticket's JSON object when the agent field is set
- [ ] `GET /api/tickets` omits `agent` from the JSON object for tickets with no agent set
- [ ] `GET /api/tickets?agent=alice` returns only tickets whose `agent` field equals `"alice"`
- [ ] `GET /api/tickets?agent=alice` excludes tickets with a different agent or no agent
- [ ] `GET /api/tickets?agent=unassigned` returns only tickets that have no agent set
- [ ] `GET /api/tickets` with no `agent` param returns all tickets regardless of agent value

### Out of scope

- Adding `agent: Option<String>` to `Frontmatter` — covered by #42f4b3ba
- CLI `--agent` filter for `apm list` — covered by #42f4b3ba
- Setting `agent` on ticket state transitions (`apm start`, `apm state in_design`, `apm take`) — covered by #42f4b3ba
- UI changes — the SupervisorView already reads `ticket.agent` and the agent filter dropdown already sends `?agent=`; no UI work needed once the API responds correctly

### Approach

All changes are in `apm-server/src/main.rs`.

**1. Extend `ListTicketsQuery` (around line 496)**

Add `agent: Option<String>` alongside the existing `author` field.

**2. Add agent filter in `list_tickets` (after the author filter block, around line 528)**

Mirror the existing `author` filter pattern. If `params.agent == "unassigned"`, retain only tickets where `fm.agent.is_none()`. Otherwise retain tickets where `fm.agent.as_deref() == Some(agent_name)`.

No changes needed to `TicketResponse` — `Frontmatter` is already `#[serde(flatten)]`-ed, so the `agent` field (added by #42f4b3ba with `skip_serializing_if = "Option::is_none"`) will appear in the response automatically when set.

**3. Tests (add to the `#[cfg(test)]` block)**

- `list_tickets_agent_field_present`: ticket with `agent = Some("alice")` — response JSON contains `"agent": "alice"`
- `list_tickets_agent_field_absent`: ticket with `agent = None` — response JSON has no `agent` key
- `list_tickets_agent_filter`: two tickets with different agents — `?agent=alice` returns only alice's ticket
- `list_tickets_agent_unassigned_filter`: one ticket with agent, one without — `?agent=unassigned` returns only the unassigned one

**Order**

1. Add `agent` to `ListTicketsQuery`
2. Add filter block in `list_tickets`
3. Add four tests
4. `cargo test --workspace` passes

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:55Z | groomed | in_design | philippepascal |