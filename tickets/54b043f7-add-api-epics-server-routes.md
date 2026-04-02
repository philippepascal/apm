+++
id = "54b043f7"
title = "Add /api/epics server routes"
state = "in_design"
priority = 4
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "52023"
branch = "ticket/54b043f7-add-api-epics-server-routes"
created_at = "2026-04-01T21:55:53.796830Z"
updated_at = "2026-04-02T00:51:44.997275Z"
+++

## Spec

### Problem

The apm-server has no API routes for epics, so the UI and external tools cannot list, create, or inspect them. Three new routes are needed.

The full design is in `docs/epics.md` (§ apm-server changes — New routes):
- `GET /api/epics` — list all epics (branch scan + derived state)
- `POST /api/epics` — create a new epic (delegates to `apm epic new`)
- `GET /api/epics/:id` — single epic detail with full ticket list

The response shape for a single epic includes `id`, `title`, `branch`, `state`, and `ticket_counts` map. The list endpoint returns `[EpicSummary]`; the detail endpoint adds a `tickets` array using the existing `TicketResponse` shape.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:51Z | groomed | in_design | philippepascal |