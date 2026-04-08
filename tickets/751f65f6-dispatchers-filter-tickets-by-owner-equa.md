+++
id = "751f65f6"
title = "Dispatchers filter tickets by owner equals current user"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/751f65f6-dispatchers-filter-tickets-by-owner-equa"
created_at = "2026-04-08T15:09:55.270545Z"
updated_at = "2026-04-08T15:09:55.270545Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["3d784167"]
+++

## Spec

### Problem

Currently `pick_next()`, `apm start --next`, and `apm work` pick up any actionable ticket regardless of owner. This allows multiple supervisors to accidentally dispatch workers on each other's tickets, and there is no record of who intended which ticket to be worked on.

Per the ownership model, dispatchers should only act on tickets owned by the current user. This means:
- A supervisor runs `apm work` → only their owned tickets are dispatched
- The UI dispatcher filters by the logged-in user's owned tickets
- `apm start --next` picks from owned tickets only

This is the key behavioral change that makes ownership meaningful.

### Acceptance criteria

- [ ] `pick_next()` in `ticket.rs` adds an owner filter: only tickets where `owner == current_user`
- [ ] `apm start --next` resolves current user and passes it to `pick_next()`
- [ ] `apm work` / `spawn_next_worker()` applies the same owner filter
- [ ] UI dispatcher applies the same filter (using the authenticated user)
- [ ] Tickets with no owner are NOT picked up by dispatchers (they must be assigned first)
- [ ] `apm next` output reflects the owner filter
- [ ] Tests: dispatcher skips unowned tickets, dispatcher skips tickets owned by others, dispatcher picks owned tickets

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
| 2026-04-08T15:09Z | — | new | philippepascal |