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

The apm-server has no API routes for epics. Clients (UI, external tooling) cannot list all epics, create a new epic, or inspect a single epic with its associated tickets.

Three routes are specified in `docs/epics.md` (§ apm-server changes — New routes):

- `GET /api/epics` — list all epics discovered from `epic/*` git branches, with derived state and per-state ticket counts
- `POST /api/epics` — create a new epic branch (`epic/<id>-<slug>`) from the tip of `origin/main`, seed it with a minimal `EPIC.md`, and push it
- `GET /api/epics/:id` — return a single epic with the same summary fields plus a full `tickets` array

Epic state is derived on demand from the states of associated tickets (those whose frontmatter contains `epic = "<id>"`). That field does not yet exist on `Frontmatter`; it must be added as a prerequisite.

### Acceptance criteria

- [ ] `GET /api/epics` returns `[]` when no `epic/*` branches exist locally or at origin
- [ ] `GET /api/epics` returns one `EpicSummary` entry per `epic/*` branch found (local or `origin/*`)
- [ ] Each `EpicSummary` contains `id`, `title`, `branch`, `state`, and `ticket_counts` fields
- [ ] `GET /api/epics` on an in-memory server returns HTTP 501
- [ ] Epic `state` is `"empty"` when no tickets carry `epic = "<id>"` in their frontmatter
- [ ] Epic `state` is `"in_progress"` when any associated ticket is in `in_design` or `in_progress`
- [ ] Epic `state` is `"implemented"` when all associated tickets are in `implemented`, `accepted`, or `closed` and at least one is in `implemented`
- [ ] Epic `state` is `"done"` when all associated tickets are in `accepted` or `closed`
- [ ] `POST /api/epics` with `{"title": "My Epic"}` returns HTTP 201 with a new `EpicSummary` (state `"empty"`, empty `ticket_counts`)
- [ ] After `POST /api/epics`, an `epic/<id>-<slug>` branch exists at origin
- [ ] `POST /api/epics` with missing or empty `title` returns HTTP 400
- [ ] `POST /api/epics` on an in-memory server returns HTTP 501
- [ ] `GET /api/epics/:id` returns the matching epic with all `EpicSummary` fields plus a `tickets` array
- [ ] Each entry in `tickets` uses the same shape as `TicketResponse` (flattened frontmatter + `body`, `has_open_questions`, `has_pending_amendments`)
- [ ] `GET /api/epics/:id` returns HTTP 404 when no `epic/*` branch whose ID segment matches `:id` exists
- [ ] `GET /api/epics/:id` on an in-memory server returns HTTP 501

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