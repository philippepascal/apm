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