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

The frontend needs read access to tickets. Add GET /api/tickets (all tickets as JSON array via ticket::load_all_from_git) and GET /api/tickets/:id (single ticket, frontmatter + body). This also validates that apm-core logic works correctly in an async axum context. Full spec context: initial_specs/UIdraft_spec_starter.md Step 2. Requires Step 1.

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