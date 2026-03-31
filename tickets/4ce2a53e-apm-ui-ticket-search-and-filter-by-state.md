+++
id = "4ce2a53e"
title = "apm-ui: ticket search and filter (by state, agent, text)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "45935"
branch = "ticket/4ce2a53e-apm-ui-ticket-search-and-filter-by-state"
created_at = "2026-03-31T06:13:17.849783Z"
updated_at = "2026-03-31T07:18:19.647756Z"
+++

## Spec

### Problem

The supervisor swimlane view (Step 5) only shows tickets grouped by supervisor-actionable states, with no way to search by text, filter to a specific state or agent, or surface closed/cancelled tickets. Supervisors who need to find a specific ticket by content, check on a specific agent's work, or review completed tickets must fall back to the CLI.

The fix is a filter bar above the swimlane grid that provides parity with the two most-used CLI flags: `apm list --state <state>` (narrow to one state, including any non-supervisor state) and `apm list --all` (expose closed/cancelled). Text search and an agent picker round out the filtering surface.

The `GET /api/tickets` response already includes a `body` field (Step 2 spec), so all filtering can run client-side against TanStack Query's cached data — no new backend endpoints are needed.

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
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:18Z | new | in_design | philippepascal |