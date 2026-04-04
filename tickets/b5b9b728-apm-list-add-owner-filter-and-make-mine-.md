+++
id = "b5b9b728"
title = "apm list: add --owner filter and make --mine match author or owner"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/b5b9b728-apm-list-add-owner-filter-and-make-mine-"
created_at = "2026-04-04T06:28:11.099983Z"
updated_at = "2026-04-04T06:51:04.244874Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

`apm list --mine` currently matches only the `author` field: it filters to tickets created by the current user. Once ticket 42f4b3ba lands and adds the `agent` ownership field to `Frontmatter`, a user who picks up a ticket created by someone else will not see it in `--mine` even though they are the active owner. The mental model of "my tickets" should include both tickets you created and tickets you are currently responsible for.

There is also no user-facing `--owner` flag to filter by who currently owns a ticket (i.e. by the `agent` field). The existing `--author` flag covers the creator dimension; the owner dimension has no equivalent.

### Acceptance criteria

- [ ] `apm list --mine` returns tickets where `author` equals the current user
- [ ] `apm list --mine` also returns tickets where `agent` equals the current user (even if `author` is someone else)
- [ ] `apm list --mine` does not return tickets where neither `author` nor `agent` matches the current user
- [ ] `apm list --owner alice` returns only tickets whose `agent` field equals `"alice"`
- [ ] `apm list --owner alice` does not return tickets authored by alice but not owned by alice
- [ ] `apm list --owner alice` with no matching tickets returns empty output and exits 0
- [ ] `--owner` and `--mine` are mutually exclusive (combining them produces an error)
- [ ] `--owner alice` and `--author bob` can be combined; both filters apply (AND logic)
- [ ] `--owner alice` and `--state ready` can be combined; both filters apply
- [ ] `apm list --help` documents the `--owner` flag

### Out of scope

- Adding the `agent` field to `Frontmatter` — handled by ticket 42f4b3ba (this ticket depends on it)
- `apm set <id> agent <name>` setter — handled by ticket 42f4b3ba
- Server API (`GET /api/tickets?owner=...`) filter — separate ticket
- UI agent/owner filter dropdown — separate ticket
- Clearing `agent` automatically on state transitions (e.g. when a ticket goes back to `specd`) — intentionally left to a future ticket
- Back-filling `agent` on existing tickets — no migration pass needed

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
| 2026-04-04T06:51Z | groomed | in_design | philippepascal |