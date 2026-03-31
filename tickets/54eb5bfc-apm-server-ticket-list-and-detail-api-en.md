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


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:13Z | new | in_design | philippepascal |