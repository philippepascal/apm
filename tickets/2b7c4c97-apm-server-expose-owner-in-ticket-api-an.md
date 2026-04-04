+++
id = "2b7c4c97"
title = "apm-server: expose owner in ticket API and add owner query param"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/2b7c4c97-apm-server-expose-owner-in-ticket-api-an"
created_at = "2026-04-04T06:28:16.243562Z"
updated_at = "2026-04-04T18:12:36.162139Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

The `/api/tickets` list endpoint does not expose the `owner` field (the person or AI agent currently assigned to work on a ticket). The `ListTicketsQuery` struct supports `?author=` filtering but has no `?owner=` query parameter.

Ticket #42f4b3ba adds `owner: Option<String>` to `Frontmatter`. Because `TicketResponse` flattens `Frontmatter` via `#[serde(flatten)]`, the field will appear in responses automatically once the dependency lands — but only when non-null (`skip_serializing_if = "Option::is_none"`). The missing piece is the server-side `?owner=` query param, which the UI's owner filter dropdown needs to perform filtered fetches rather than client-side filtering over the full ticket list.

### Acceptance criteria

- [x] `GET /api/tickets` includes `owner` in each ticket's JSON object when the owner field is set
- [x] `GET /api/tickets` omits `owner` from the JSON object for tickets with no owner set
- [x] `GET /api/tickets?owner=alice` returns only tickets whose `owner` field equals `"alice"`
- [x] `GET /api/tickets?owner=alice` excludes tickets with a different owner or no owner
- [x] `GET /api/tickets?owner=unassigned` returns only tickets that have no owner set
- [x] `GET /api/tickets` with no `owner` param returns all tickets regardless of owner value

### Out of scope

- Adding `owner: Option<String>` to `Frontmatter` — covered by #42f4b3ba
- CLI `--owner` filter for `apm list` — covered by #42f4b3ba
- Setting `owner` on ticket state transitions (`apm start`, `apm state in_design`, `apm take`) — covered by #42f4b3ba
- UI changes — the SupervisorView already reads `ticket.owner` and the owner filter dropdown already sends `?owner=`; no UI work needed once the API responds correctly

### Approach

All changes are in `apm-server/src/main.rs`.

**1. Extend `ListTicketsQuery` (around line 496)**

Add `owner: Option<String>` alongside the existing `author` field.

**2. Add owner filter in `list_tickets` (after the author filter block, around line 528)**

Mirror the existing `author` filter pattern. If `params.owner == "unassigned"`, retain only tickets where `fm.owner.is_none()`. Otherwise retain tickets where `fm.owner.as_deref() == Some(owner_name)`.

No changes needed to `TicketResponse` — `Frontmatter` is already `#[serde(flatten)]`-ed, so the `owner` field (added by #42f4b3ba with `skip_serializing_if = "Option::is_none"`) will appear in the response automatically when set.

**3. Tests (add to the `#[cfg(test)]` block)**

- `list_tickets_owner_field_present`: ticket with `owner = Some("alice")` — response JSON contains `"owner": "alice"`
- `list_tickets_owner_field_absent`: ticket with `owner = None` — response JSON has no `owner` key
- `list_tickets_owner_filter`: two tickets with different owners — `?owner=alice` returns only alice's ticket
- `list_tickets_owner_unassigned_filter`: one ticket with owner, one without — `?owner=unassigned` returns only the unassigned one

**Order**

1. Add `owner` to `ListTicketsQuery`
2. Add filter block in `list_tickets`
3. Add four tests
4. `cargo test --workspace` passes

### Open questions


### Amendment requests

- [x] Rename `agent` to `owner` throughout: query param is `?owner=` (not `?agent=`), `ListTicketsQuery` field is `owner: Option<String>`, filter logic uses `fm.owner`, JSON response field is `owner`
- [x] Update acceptance criteria: `?agent=alice` → `?owner=alice`, `?agent=unassigned` → `?owner=unassigned`
- [x] Update test names: `list_tickets_agent_*` → `list_tickets_owner_*`
- [x] Update out-of-scope: references to `agent` field → `owner` field

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:55Z | groomed | in_design | philippepascal |
| 2026-04-04T06:58Z | in_design | specd | claude-0403-0700-b2c7 |
| 2026-04-04T07:15Z | specd | ammend | apm |
| 2026-04-04T07:15Z | ammend | in_design | philippepascal |
| 2026-04-04T07:17Z | in_design | specd | claude-0404-0715-spec1 |
| 2026-04-04T15:33Z | specd | ready | apm |
| 2026-04-04T17:00Z | ready | in_progress | philippepascal |
| 2026-04-04T17:04Z | in_progress | implemented | claude-0404-1700-w2b7 |
| 2026-04-04T18:12Z | implemented | closed | apm-sync |
