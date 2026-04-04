+++
id = "9931e70f"
title = "Queue: exclude tickets owned by another user"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/9931e70f-queue-exclude-tickets-owned-by-another-u"
created_at = "2026-04-04T06:28:25.839773Z"
updated_at = "2026-04-04T07:02:33.789551Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["ffaad988"]
+++

## Spec

### Problem

The priority queue (`/api/queue` and `apm next`) shows all tickets actionable by an agent, regardless of who owns them. Since owner persists for the entire ticket lifecycle, a `ready` ticket owned by Alice shouldn't appear in Bob's queue — Alice owns it and will pick it back up. The queue should exclude tickets where `owner` is set to someone other than the requesting user. Unowned tickets remain visible to everyone.

### Acceptance criteria

- [ ] `apm next` does not return a ticket whose `agent` field is set to a user other than the running agent
- [ ] `apm next` returns a ticket whose `agent` field matches the running agent (owner resuming their own work)
- [ ] `apm next` returns a ticket with no `agent` field set
- [ ] `apm start --next` does not pick a ticket owned by a different user
- [ ] `GET /api/queue` excludes tickets whose `agent` differs from the authenticated caller
- [ ] `GET /api/queue` includes tickets with no `agent` set
- [ ] `GET /api/queue` includes tickets whose `agent` matches the authenticated caller
- [ ] When the caller cannot be determined (no session, no localhost identity), `/api/queue` returns all tickets unchanged (no ownership filter applied)

### Out of scope

- Filtering `apm list` by ownership (that filter is `--agent` and is covered by ticket 42f4b3ba)
- Enforcing ownership at write time (tickets can still be started by anyone; the queue filter is advisory, not a lock)
- Adding auth to the `/api/queue` endpoint (authentication is handled by the broader user-mgmt epic)
- Clearing `agent` on any state transition (ownership is sticky by design, per ticket ffaad988)
- Back-filling ownership on existing tickets

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
| 2026-04-04T07:02Z | groomed | in_design | philippepascal |