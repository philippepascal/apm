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

Building new collaborator validation — that is covered by tickets bbd5d271 and c738d9cc.

### Approach

1. Delete `resolve_collaborators()` and its tests from `apm-core/src/config.rs`.
2. Delete `fetch_repo_collaborators()` from `apm-core/src/github.rs` if only used by the above.
3. Audit `resolve_agent_name()` call sites in `start.rs` — ensure it is only used for `append_history()` and worker spawning env vars, never for ownership checks or ticket filtering.
4. Add a doc comment on `resolve_agent_name()` clarifying: "Returns the name recorded in ticket history. This is NOT the ticket owner."

See `docs/ownership-spec.md` for the full ownership model.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |