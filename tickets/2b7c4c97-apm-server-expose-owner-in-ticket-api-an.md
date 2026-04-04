+++
id = "2b7c4c97"
title = "apm-server: expose owner in ticket API and add owner query param"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/2b7c4c97-apm-server-expose-owner-in-ticket-api-an"
created_at = "2026-04-04T06:28:16.243562Z"
updated_at = "2026-04-04T06:55:00.033254Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

The `/api/tickets` list endpoint does not expose the `agent` field (the person or AI agent currently assigned to work on a ticket). The `ListTicketsQuery` struct supports `?author=` filtering but has no `?agent=` query parameter.

Ticket #42f4b3ba adds `agent: Option<String>` to `Frontmatter`. Because `TicketResponse` flattens `Frontmatter` via `#[serde(flatten)]`, the field will appear in responses automatically once the dependency lands — but only when non-null (`skip_serializing_if = "Option::is_none"`). The missing piece is the server-side `?agent=` query param, which the UI's agent filter dropdown needs to perform filtered fetches rather than client-side filtering over the full ticket list.

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
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
| 2026-04-04T06:55Z | groomed | in_design | philippepascal |