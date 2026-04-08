+++
id = "751f65f6"
title = "Dispatchers filter tickets by owner equals current user"
state = "in_design"
priority = 0
effort = 4
risk = 0
author = "philippepascal"
branch = "ticket/751f65f6-dispatchers-filter-tickets-by-owner-equa"
created_at = "2026-04-08T15:09:55.270545Z"
updated_at = "2026-04-08T16:02:09.032665Z"
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

**Background on existing code:** `sorted_actionable()` in `ticket.rs` already uses the `caller` parameter for a partial owner filter — it includes unowned tickets, excludes tickets owned by others. But `caller` is the *agent name* (e.g. `"claude"`), not the supervisor identity. This means tickets owned by a supervisor (e.g. `"alice"`) are already excluded when an agent runs dispatch, but the semantics are wrong: unowned tickets still get dispatched, and the supervisor identity is never used. This ticket fixes both issues.

**Step 1 — `apm-core/src/ticket.rs`: replace caller-based owner filter with explicit `owner_filter`**

- Add `owner_filter: Option<&str>` as a new parameter to `sorted_actionable()` and `pick_next()`.
- Remove the existing owner-matching code from `sorted_actionable()` that uses `caller` (the `None => true / Some(owner) => caller.map_or(true, |c| c == owner)` filter). This is being replaced, not extended.
- Add a new filter clause: when `owner_filter = Some(user)`, only include tickets where `frontmatter.owner == Some(user)`. Unowned tickets are excluded.
- When `owner_filter = None`, no owner filtering is applied (all tickets pass through).
- The existing `caller` parameter stays — its role is to identify the calling agent for any future agent-name-specific logic; it no longer drives ownership filtering.
- Update all internal call sites of `sorted_actionable()` and `pick_next()` to pass `owner_filter` (pass `None` for non-dispatcher callers that don't need ownership gating).

**Step 2 — `apm-core/src/start.rs::run_next()` (backs `apm start --next`)**

- After loading config, call `let current_user = resolve_identity(root);`.
- Pass `Some(current_user.as_str())` as `owner_filter` to `pick_next()`.

**Step 3 — `apm-core/src/start.rs::spawn_next_worker()` (backs `apm work` and the server engine loop)**

- After loading config, call `let current_user = resolve_identity(root);`.
- Pass `Some(current_user.as_str())` as `owner_filter` to `pick_next()`.
- No signature change to `spawn_next_worker()` needed; `resolve_identity` is available in `apm_core`.

**Step 4 — `apm/src/cmd/next.rs` (backs `apm next`)**

- After loading config, call `let current_user = apm_core::config::resolve_identity(root);`.
- Pass `Some(current_user.as_str())` as `owner_filter` to `pick_next()`.

**Step 5 — `apm-server/src/work.rs::get_work_dry_run()`**

- Inside the `spawn_blocking` closure, after loading config, call `let current_user = apm_core::config::resolve_identity(&root);`.
- Add an owner filter to the existing `filtered` vec: `filtered.retain(|t| t.frontmatter.owner.as_deref() == Some(current_user.as_str()));`
- This endpoint does not use `pick_next()`, so it needs an inline filter.

**Step 6 — Tests in `apm-core/src/ticket.rs`**

- Update `sorted_actionable_includes_unowned_ticket`: change assertion so that unowned tickets are *excluded* when an `owner_filter` is supplied.
- Update `sorted_actionable_no_caller_shows_all`: when `owner_filter = None`, all tickets still pass through.
- Add `pick_next_skips_unowned_ticket_when_owner_filter_set`.
- Add `pick_next_skips_ticket_owned_by_other`.
- Add `pick_next_picks_ticket_owned_by_current_user`.
- The `sorted_actionable_excludes_ticket_owned_by_other` and `sorted_actionable_includes_ticket_owned_by_caller` tests from the prior ticket (`3d784167`) will need their call signatures updated to use `owner_filter` instead of `caller`.

**Note:** `docs/ownership-spec.md` is referenced in the original draft but does not exist. Ignore that reference.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:56Z | groomed | in_design | philippepascal |