+++
id = "919412f4"
title = "Owner immutable after ticket is closed"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
branch = "ticket/919412f4-owner-immutable-after-ticket-is-closed"
created_at = "2026-04-08T15:09:50.464294Z"
updated_at = "2026-04-08T23:47:16.670582Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

Closed tickets should be immutable records. Changing the owner of a closed ticket has no practical purpose and could corrupt the audit trail. The ownership check should reject owner changes on tickets in a terminal state.

### Acceptance criteria

- [x] `apm assign <id> <user>` on a closed ticket fails with "cannot change owner of a closed ticket"
- [x] `apm set <id> owner <user>` on a closed ticket fails with same error
- [x] The check uses the workflow config `terminal` flag on the ticket's current state
- [x] Tests cover: owner change rejected on closed ticket, allowed on non-terminal states

### Out of scope

Preventing other field changes on closed tickets (only owner is gated here).

### Approach

This ticket extends the `check_owner()` helper introduced by b0708201. All work is in `apm-core/src/ticket.rs` plus tests; no changes to `assign.rs` or `set.rs` are needed (they already call `check_owner()` after b0708201).

**`apm-core/src/ticket.rs` — extend `check_owner()`**

`check_owner(root: &Path, ticket: &Ticket) -> Result<()>` (added by b0708201) checks that the caller's resolved identity matches `ticket.frontmatter.owner`. Add a terminal-state guard at the top of the function, before the identity check:

```rust
// Reject owner changes on terminal states
let cfg = Config::load(root)?;
let is_terminal = cfg.workflow.states.iter()
    .find(|s| s.id == ticket.frontmatter.state)
    .map(|s| s.terminal)
    .unwrap_or(false);
if is_terminal {
    anyhow::bail!("cannot change owner of a closed ticket");
}
```

`Config::load` is already available in `apm-core`. `ticket.frontmatter.state` holds the current state string.

**Order of checks inside `check_owner()`:**
1. Terminal-state guard (this ticket) — short-circuits immediately; no identity resolution needed.
2. Identity resolution and ownership comparison (b0708201).

**Tests — add alongside b0708201's `check_owner` tests in `apm-core/src/ticket.rs` or `apm/tests/integration.rs`**

Follow the tempfile + config-writing pattern used in existing ownership and config tests:

- `check_owner_rejects_owner_change_on_terminal_state`: write a minimal workflow config with state `"closed"` where `terminal = true`; create a ticket in that state; assert `check_owner()` errors with `"cannot change owner of a closed ticket"`.
- `check_owner_allows_owner_change_on_non_terminal_state`: same setup but ticket is in `"open"` (`terminal = false`); owner matches current identity; assert `check_owner()` returns `Ok(())`.

**No changes required to:**
- `assign.rs` — already calls `check_owner()` after b0708201
- `set.rs` — already guards `field == "owner"` and calls `check_owner()` after b0708201
- `config.rs` — `terminal: bool` already exists on `StateConfig`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:52Z | groomed | in_design | philippepascal |
| 2026-04-08T15:55Z | in_design | specd | claude-0408-1552-3420 |
| 2026-04-08T21:47Z | specd | ready | apm |
| 2026-04-08T22:10Z | ready | in_progress | philippepascal |
| 2026-04-08T22:15Z | in_progress | implemented | claude-0408-2210-05c0 |
| 2026-04-08T23:47Z | implemented | closed | apm-sync |
