+++
id = "b0708201"
title = "Only owner can change owner"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/b0708201-only-owner-can-change-owner"
created_at = "2026-04-08T15:09:45.724421Z"
updated_at = "2026-04-08T15:09:45.724421Z"
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