+++
id = "919412f4"
title = "Owner immutable after ticket is closed"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/919412f4-owner-immutable-after-ticket-is-closed"
created_at = "2026-04-08T15:09:50.464294Z"
updated_at = "2026-04-08T15:33:39.679158Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

Closed tickets should be immutable records. Changing the owner of a closed ticket has no practical purpose and could corrupt the audit trail. The ownership check should reject owner changes on tickets in a terminal state.

### Acceptance criteria

- [ ] `apm assign <id> <user>` on a closed ticket fails with "cannot change owner of a closed ticket"
- [ ] `apm set <id> owner <user>` on a closed ticket fails with same error
- [ ] The check uses the workflow config `terminal` flag on the ticket's current state
- [ ] Tests cover: owner change rejected on closed ticket, allowed on non-terminal states

### Out of scope

Preventing other field changes on closed tickets (only owner is gated here).

### Approach

Add a terminal-state check in the `check_owner()` helper (from ticket b0708201) before the ownership comparison. Load the workflow config, find the state config for the ticket's current state, and reject if `terminal == true`. See `docs/ownership-spec.md`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
