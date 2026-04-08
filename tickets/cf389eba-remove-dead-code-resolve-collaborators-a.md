+++
id = "cf389eba"
title = "Remove dead code: resolve_collaborators and agent_name ownership overlap"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/cf389eba-remove-dead-code-resolve-collaborators-a"
created_at = "2026-04-08T15:09:36.685009Z"
updated_at = "2026-04-08T15:42:27.925830Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

The codebase has two dead-code problems and one naming/documentation problem that together confuse the ownership model.\n\n1. **`resolve_collaborators()` is dead at runtime.** The function in `apm-core/src/config.rs` fetches GitHub collaborators or falls back to static config, but is never called outside its own test module. It gives the impression that collaborator resolution is active when it is not.\n\n2. **`fetch_repo_collaborators()` is also dead.** The function in `apm-core/src/github.rs` is only called by `resolve_collaborators()` (plus one `#[ignore]`-d live test). It becomes unreachable once `resolve_collaborators()` is removed.\n\n3. **`resolve_agent_name()` is misnamed and under-documented.** The function in `apm-core/src/start.rs` is used in two distinct roles: (a) recording the acting party in ticket history via `append_history()`, and (b) supplying the `caller` argument to `pick_next()` / `sorted_actionable()`, which filters tickets by comparing caller identity against the ticket `owner` field. The name "agent_name" implies a transient worker concept and hides the fact that the same identity drives owner-based filtering — a source of confusion between "who is logged as acting" and "who is allowed to pick a ticket".

### Acceptance criteria

- [ ] `resolve_collaborators()` and both its tests removed from `apm-core/src/config.rs`\n- [ ] `fetch_repo_collaborators()` and its live `#[ignore]` test removed from `apm-core/src/github.rs`\n- [ ] `resolve_agent_name()` renamed to `resolve_caller_name()` (or equivalent) across all call sites\n- [ ] The renamed function has a doc comment that states: it returns the identity used (a) to record history entries and (b) as the caller when filtering tickets by `owner`\n- [ ] No remaining code refers to the old `resolve_agent_name` name\n- [ ] All existing tests pass

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
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:42Z | groomed | in_design | philippepascal |