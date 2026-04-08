+++
id = "b52fc7f4"
title = "Bulk owner change for epic tickets"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/b52fc7f4-bulk-owner-change-for-epic-tickets"
created_at = "2026-04-08T15:10:08.148508Z"
updated_at = "2026-04-08T15:10:08.148508Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

When a supervisor wants to hand off an entire epic to another supervisor, they must change the owner on each ticket individually. This is tedious for epics with many tickets. A convenience command should change the owner of all non-closed tickets in an epic at once.

### Acceptance criteria

- [ ] `apm epic set <epic-id> owner <user>` changes owner on all non-closed tickets in the epic
- [ ] Closed (terminal) tickets are skipped
- [ ] Output lists each ticket that was changed and any that were skipped
- [ ] The ownership check applies: current user must be the owner of each ticket being changed
- [ ] If any ticket fails the ownership check, none are changed (atomic: all or nothing)
- [ ] Owner validation (collaborator check) applies to the new owner
- [ ] Tests cover: bulk change succeeds, closed tickets skipped, non-owner blocked

### Out of scope

Bulk owner change across multiple epics. Changing owner of the epic itself (epics do not have owners).

### Approach

Extend `apm epic set` (in `apm/src/cmd/epic.rs` `run_set()`) to handle "owner" as a field. Load all tickets for the epic, filter out terminal-state tickets, run the ownership check on each, validate the new owner, then update each ticket's owner field on its branch. If any check fails, abort before making changes. See `docs/ownership-spec.md`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:10Z | — | new | philippepascal |