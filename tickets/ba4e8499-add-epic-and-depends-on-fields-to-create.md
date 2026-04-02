+++
id = "ba4e8499"
title = "Add epic and depends_on fields to CreateTicketRequest and ticket API responses"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "85628"
branch = "ticket/ba4e8499-add-epic-and-depends-on-fields-to-create"
created_at = "2026-04-01T21:55:57.801343Z"
updated_at = "2026-04-02T00:43:44.702702Z"
+++

## Spec

### Problem

The `CreateTicketRequest` struct in `apm-server/src/main.rs` accepts only `title` and `sections`. It has no `epic` or `depends_on` fields, so the UI cannot create epic-linked or dependency-declared tickets via the API.

The `Frontmatter` struct in `apm-core/src/ticket.rs` also has no `epic`, `target_branch`, or `depends_on` fields. Because `TicketResponse` and `TicketDetailResponse` both serialize frontmatter via `#[serde(flatten)]`, adding these fields to `Frontmatter` is sufficient to make them appear in all existing ticket API read responses — no struct changes are needed in `apm-server`.

The `ticket::create` function must also be extended to accept and persist these three optional fields so the server (and the CLI in a future ticket) can populate them at creation time.

### Acceptance criteria

- [ ] `GET /api/tickets` response includes `epic`, `target_branch`, and `depends_on` keys for a ticket that has those frontmatter fields set
- [ ] `GET /api/tickets/:id` response includes `epic`, `target_branch`, and `depends_on` keys for a ticket that has those frontmatter fields set
- [ ] `GET /api/tickets` response omits `epic`, `target_branch`, and `depends_on` for a ticket that does not have those fields (keys absent, not null)
- [ ] `POST /api/tickets` with `{"title": "T", "depends_on": ["ab12cd34"]}` creates a ticket whose frontmatter contains `depends_on = ["ab12cd34"]`
- [ ] `POST /api/tickets` with `{"title": "T", "epic": "ab12cd34"}` where branch `epic/ab12cd34-some-slug` exists creates a ticket with `epic = "ab12cd34"` and `target_branch = "epic/ab12cd34-some-slug"` in frontmatter
- [ ] `POST /api/tickets` with `{"title": "T", "epic": "ab12cd34"}` where no matching epic branch exists returns HTTP 400
- [ ] `POST /api/tickets` response body includes `epic`, `target_branch`, and `depends_on` when those fields were set
- [ ] Existing `POST /api/tickets` calls with no `epic` or `depends_on` fields continue to work unchanged

### Out of scope

- Epic CRUD routes (`GET /api/epics`, `POST /api/epics`, `GET /api/epics/:id`) — separate ticket
- `apm epic new` / `apm epic list` / `apm epic show` CLI commands — separate ticket
- `apm new --epic` CLI flag — separate ticket
- `depends_on` scheduling in the engine loop — separate ticket
- UI changes (epic dropdown in new-ticket modal, lock icons on cards, epic filter) — separate ticket
- `POST /api/work/start` epic filter field — separate ticket
- `apm start` worktree provisioning from `target_branch` — separate ticket

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
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |