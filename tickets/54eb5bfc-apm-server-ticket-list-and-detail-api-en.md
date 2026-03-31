+++
id = "54eb5bfc"
title = "apm-server: ticket list and detail API endpoints"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "38135"
branch = "ticket/54eb5bfc-apm-server-ticket-list-and-detail-api-en"
created_at = "2026-03-31T06:11:28.689659Z"
updated_at = "2026-03-31T06:13:41.748340Z"
+++

## Spec

### Problem

The frontend needs read access to ticket data served over HTTP. Currently there is no API layer — only the CLI and the underlying `apm-core` library. Adding `GET /api/tickets` and `GET /api/tickets/:id` endpoints to the axum server (scaffolded in Step 1) gives the frontend a stable JSON interface to list all tickets and inspect individual ones. It also validates that `apm-core`'s synchronous git-reading functions integrate cleanly with axum's async runtime without blocking the event loop.

### Acceptance criteria

- [ ] `GET /api/tickets` returns HTTP 200 with `Content-Type: application/json`
- [ ] The response body is a JSON array where each element contains all frontmatter fields plus a `body` string
- [ ] `GET /api/tickets/:id` with a valid ticket ID prefix returns HTTP 200 with a JSON object for that ticket
- [ ] `GET /api/tickets/:id` with an unknown ID returns HTTP 404
- [ ] `GET /api/tickets/:id` accepts a 4–8 hex-char prefix or a zero-padded integer (same matching rules as `apm show`)
- [ ] The server does not block the tokio runtime while reading from git (blocking work is off-loaded via spawn_blocking)

### Out of scope

- Write/mutation endpoints (covered by later steps: state transition, body edit, ticket create)
- Authentication or authorization
- Pagination, sorting, or filtering of the ticket list
- The React/Vite frontend that consumes these endpoints (Step 3)
- Worker, sync, or state-transition endpoints
- The apm-server crate scaffold itself (Step 1 prerequisite)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:13Z | new | in_design | philippepascal |