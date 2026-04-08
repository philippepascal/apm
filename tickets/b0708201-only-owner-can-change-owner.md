+++
id = "b0708201"
title = "Only owner can change owner"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "philippepascal"
branch = "ticket/b0708201-only-owner-can-change-owner"
created_at = "2026-04-08T15:09:45.724421Z"
updated_at = "2026-04-08T15:52:12.677462Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

Currently anyone can change a ticket's owner via `apm assign` or `apm set owner` with no checks. The ownership model requires that only the current owner can reassign ownership. This prevents accidental or unauthorized reassignment and creates a clear audit trail of ownership transfers. The check uses `resolve_identity()` to determine the current user and compares against the ticket's owner field.

### Acceptance criteria

- [ ] `apm assign <id> <user>` checks that current user == ticket owner before changing
- [ ] If current user != owner, command fails with a clear error: "only the current owner (<owner>) can reassign this ticket"
- [ ] `apm set <id> owner <user>` has the same check
- [ ] The check uses `resolve_identity()` (respects config-based vs GitHub mode)
- [ ] If identity cannot be resolved (returns "unassigned"), the command fails with an error asking to configure identity
- [ ] Tests cover: owner can reassign, non-owner is rejected, unresolved identity is rejected

### Out of scope

Validation that the new owner is a valid collaborator (separate tickets bbd5d271, c738d9cc). Terminal state check (separate ticket 919412f4).

### Approach

1. Add a helper `check_owner(root: &Path, ticket: &Ticket) -> Result<()>` in `apm-core/src/ticket.rs` (alongside `set_field`):
   - Calls `resolve_identity(root)` to get the current user as a `String`
   - If identity resolves to `"unassigned"`, bail: `"cannot reassign: identity not configured (set local.user in .apm/local.toml or configure a GitHub token)"`
   - Compares result against `ticket.frontmatter.owner` (an `Option<String>`):
     - If `owner` is `None`, anyone can claim — no check needed (unowned ticket); return `Ok(())`
     - If `owner` is `Some(o)` and current user != `o`, bail: `"only the current owner ({o}) can reassign this ticket"`
   - Returns `Ok(())` if the check passes

2. Call `check_owner(root, &t)` in `apm/src/cmd/assign.rs` immediately before the `ticket::set_field()` call (currently line 29). The `root` path is already available in the function signature.

3. Call `check_owner(root, &t)` in `apm/src/cmd/set.rs` when the field being set is `"owner"`, before the `ticket::set_field()` call. Add an `if field == "owner"` guard around it. The `CmdContext` already carries `root`.

4. Do **not** add the check inside `ticket::set_field()` — that function only receives `&mut Frontmatter` and has no access to repo root or identity resolution.

5. Add unit tests in `apm-core` (following the tempfile pattern used in `config.rs` tests):
   - Owner matches current user → `Ok(())`
   - Owner differs from current user → error containing the owner's name
   - Identity resolves to `"unassigned"` → error asking to configure identity
   - Ticket has no owner (`None`) → `Ok(())` (open for claiming)

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:49Z | groomed | in_design | philippepascal |