+++
id = "cf389eba"
title = "Remove dead code: resolve_collaborators and agent_name ownership overlap"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/cf389eba-remove-dead-code-resolve-collaborators-a"
created_at = "2026-04-08T15:09:36.685009Z"
updated_at = "2026-04-08T15:09:36.685009Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

The codebase has dead code that confuses the ownership model: (1) `resolve_collaborators()` in config.rs is defined and tested but never called at runtime. (2) `resolve_agent_name()` in start.rs is used for history/logging but its name suggests an ownership concept — it should be clearly scoped to history only. (3) The `agent` concept in ticket history overlaps conceptually with `owner`, creating confusion about who is responsible for a ticket vs who is working on it.

### Acceptance criteria

- [ ] `resolve_collaborators()` removed from `config.rs` (will be replaced by active validation in a later ticket)
- [ ] `resolve_agent_name()` renamed or documented to clarify it is for history/logging only, not ownership
- [ ] No code path uses agent_name as an ownership or filtering concept
- [ ] Tests for removed functions cleaned up
- [ ] All remaining tests pass

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