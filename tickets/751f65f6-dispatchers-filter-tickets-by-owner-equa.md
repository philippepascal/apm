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

Filtering `apm list` by owner (already exists as --owner flag). Role-based filtering (agent vs supervisor actionability).

### Approach

1. Add `owner_filter: Option<&str>` parameter to `pick_next()` / `sorted_actionable()` in `apm-core/src/ticket.rs`. Filter tickets where `frontmatter.owner == Some(owner_filter)`.
2. In `apm-core/src/start.rs` `spawn_next_worker()`, resolve current user via `resolve_identity()` and pass as owner filter.
3. In `apm/src/cmd/start.rs` (CLI `apm start --next`), same approach.
4. In `apm-core/src/work.rs` dispatch loop, same approach.
5. In `apm-server` dispatcher endpoint, use the authenticated user as the owner filter.
6. Existing `caller` parameter in `pick_next()` (used for agent_name matching on already-started tickets) should be kept separate from the new owner filter.

See `docs/ownership-spec.md` for the full ownership model.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |