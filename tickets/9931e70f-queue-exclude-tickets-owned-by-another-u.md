+++
id = "9931e70f"
title = "Queue: exclude tickets owned by another user"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/9931e70f-queue-exclude-tickets-owned-by-another-u"
created_at = "2026-04-04T06:28:25.839773Z"
updated_at = "2026-04-04T06:35:26.548582Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["ffaad988"]
+++

## Spec

### Problem

The priority queue (`/api/queue` and `apm next`) shows all tickets actionable by an agent, regardless of who owns them. Since owner persists for the entire ticket lifecycle, a `ready` ticket owned by Alice shouldn't appear in Bob's queue — Alice owns it and will pick it back up. The queue should exclude tickets where `owner` is set to someone other than the requesting user. Unowned tickets remain visible to everyone.

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
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
