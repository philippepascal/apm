+++
id = "e8a56566"
title = "UI supervisor list APIs should not pull closed ticket by default"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm-ui"
agent = "33721"
branch = "ticket/e8a56566-ui-supervisor-list-apis-should-not-pull-"
created_at = "2026-04-02T18:12:19.697833Z"
updated_at = "2026-04-02T20:43:19.480949Z"
+++

## Spec

### Problem

The `GET /api/tickets` endpoint currently returns all tickets unconditionally — including tickets in the `closed` state. The UI does all filtering client-side after receiving the full payload. Because closed tickets accumulate over time and can far outnumber active tickets, this causes progressively heavier API responses and slows the UI's fast-refresh polling loop.

The desired behaviour is that the server excludes closed (terminal) tickets from the default response, and only includes them when the caller explicitly opts in. This mirrors how the CLI already works: `apm list` hides terminal states unless `--all` is passed.

### Acceptance criteria

- [x] `GET /api/tickets` without query parameters returns only non-closed tickets
- [x] `GET /api/tickets?include_closed=true` returns all tickets including closed ones
- [x] The UI supervisor view passes `include_closed=true` to the API when the "Show closed" checkbox is checked
- [x] The UI supervisor view does not pass `include_closed=true` (or omits the parameter) when the checkbox is unchecked
- [x] Closed tickets do not appear in the default supervisor list view on page load
- [x] Toggling "Show closed" on re-fetches the ticket list and closed tickets appear
- [x] Toggling "Show closed" off re-fetches the ticket list and closed tickets disappear

### Out of scope

- Adding other server-side filter query parameters (state, agent, epic, etc.) — client-side filtering for those remains unchanged
- Pagination or cursor-based loading of tickets
- Any changes to the CLI `apm list` command (it already handles this correctly)
- Caching or other performance optimisations beyond the closed-ticket exclusion
- The `GET /api/tickets/:id` single-ticket endpoint (already filtered to a specific ticket)

### Approach

**Backend — apm-server/src/main.rs**

1. Add a query-string extractor struct to the `list_tickets` handler:

   ```rust
   #[derive(Deserialize, Default)]
   struct ListTicketsQuery {
       include_closed: Option<bool>,
   }
   ```

2. Change the handler signature to also accept `Query(params): Query<ListTicketsQuery>`.

3. After loading all tickets, filter before constructing `TicketResponse` objects:
   - If `params.include_closed` is not `Some(true)`, skip tickets whose state is terminal.
   - Use the existing `apm_core::ticket::list_filtered` with `all = params.include_closed.unwrap_or(false)` and all other filter params set to defaults (`state_filter: None`, `unassigned: false`, `supervisor_filter: None`, `actionable_filter: None`).

**Frontend — apm-ui/src/components/supervisor/SupervisorView.tsx**

1. Append `?include_closed=true` to the `/api/tickets` fetch URL when `showClosed` is `true`.
2. Add `showClosed` as a dependency to the effect/query that drives the ticket fetch so toggling the checkbox triggers a re-fetch.
3. The existing client-side guard that adds `'closed'` to `visibleStates` can remain; it becomes a no-op when the server already excludes closed tickets by default.

**Order of changes**

1. Backend first (backward-compatible: callers get fewer results by default).
2. Frontend second (update fetch URL + dependency array).
3. Add a server-side test verifying: default response omits closed tickets; `?include_closed=true` includes them.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T18:12Z | — | new | apm-ui |
| 2026-04-02T18:12Z | new | groomed | apm |
| 2026-04-02T18:13Z | groomed | in_design | philippepascal |
| 2026-04-02T18:16Z | in_design | specd | claude-0402-1813-s9w1 |
| 2026-04-02T19:09Z | specd | ready | apm |
| 2026-04-02T19:27Z | ready | in_progress | philippepascal |
| 2026-04-02T19:34Z | in_progress | implemented | claude-0402-1927-e9b8 |
| 2026-04-02T20:43Z | implemented | closed | apm-sync |