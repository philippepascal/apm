+++
id = "cf389eba"
title = "Remove dead code: resolve_collaborators and agent_name ownership overlap"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
branch = "ticket/cf389eba-remove-dead-code-resolve-collaborators-a"
created_at = "2026-04-08T15:09:36.685009Z"
updated_at = "2026-04-08T21:47:35.308309Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

The codebase has two dead-code problems and one naming/documentation problem that together confuse the ownership model.\n\n1. **`resolve_collaborators()` is dead at runtime.** The function in `apm-core/src/config.rs` fetches GitHub collaborators or falls back to static config, but is never called outside its own test module. It gives the impression that collaborator resolution is active when it is not.\n\n2. **`fetch_repo_collaborators()` is also dead.** The function in `apm-core/src/github.rs` is only called by `resolve_collaborators()` (plus one `#[ignore]`-d live test). It becomes unreachable once `resolve_collaborators()` is removed.\n\n3. **`resolve_agent_name()` is misnamed and under-documented.** The function in `apm-core/src/start.rs` is used in two distinct roles: (a) recording the acting party in ticket history via `append_history()`, and (b) supplying the `caller` argument to `pick_next()` / `sorted_actionable()`, which filters tickets by comparing caller identity against the ticket `owner` field. The name "agent_name" implies a transient worker concept and hides the fact that the same identity drives owner-based filtering — a source of confusion between "who is logged as acting" and "who is allowed to pick a ticket".

### Acceptance criteria

- [x] `resolve_collaborators()` and both its tests removed from `apm-core/src/config.rs`
- [x] `fetch_repo_collaborators()` and its live `#[ignore]` test removed from `apm-core/src/github.rs`
- [x] `resolve_agent_name()` renamed to `resolve_caller_name()` (or equivalent) across all call sites
- [x] The renamed function has a doc comment that states: it returns the identity used (a) to record history entries and (b) as the caller when filtering tickets by `owner`
- [x] No remaining code refers to the old `resolve_agent_name` name
- [x] All existing tests pass

### Out of scope

- Building new collaborator validation — covered by tickets bbd5d271 and c738d9cc\n- Changing the owner-based filtering behaviour of `pick_next()` / `sorted_actionable()` — this ticket only clarifies naming, not logic\n- Creating `docs/ownership-spec.md` — that document does not yet exist; the full ownership model is defined in a later ticket

### Approach

**1. Remove `resolve_collaborators()` from `apm-core/src/config.rs`**
- Delete the `pub fn resolve_collaborators` function (lines ~430-443)
- Delete both tests: `resolve_collaborators_returns_static_when_no_git_host` and `resolve_collaborators_returns_static_when_github_but_no_token` (lines ~1052-1087)
- Remove any `pub use` re-export of this function if present

**2. Remove `fetch_repo_collaborators()` from `apm-core/src/github.rs`**
- Confirm no other call site remains by grepping for `fetch_repo_collaborators` before deleting
- Delete the `pub fn fetch_repo_collaborators` function (lines ~34-55)
- Delete the `#[ignore]` live test `fetch_repo_collaborators_live` (~line 74)

**3. Rename `resolve_agent_name()` → `resolve_caller_name()` in `apm-core/src/start.rs`**
- Rename the function definition at line ~62
- Update all call sites:
  - `start.rs:237` — history logging path
  - `start.rs:313` and `start.rs:366` — `StartOutput` struct initialization
  - `start.rs:390` — passed to `pick_next()` in `run_next()`
  - `start.rs:559` — passed to `pick_next()` in `pick_epic_ticket()`
  - `apm/src/cmd/next.rs:19` — passed to `pick_next()`
  - `apm/src/main.rs:753` — start command handler
  - Test call sites in start.rs (~lines 986, 996, 1006)

**4. Add doc comment to `resolve_caller_name()`**

```rust
/// Returns the caller identity for this process.
///
/// This value is used in two places:
/// - Recorded as the acting party in ticket history entries.
/// - Compared against a ticket's `owner` field when filtering candidates
///   in `pick_next()` / `sorted_actionable()`. Tickets owned by another
///   identity are excluded from the pick set.
///
/// Resolution order: `APM_AGENT_NAME` env var → `USER` → `USERNAME` → `"apm"`.
```

**5. Verify**
- `cargo test` — all tests green
- `grep -r resolve_agent_name .` — no matches
- `grep -r fetch_repo_collaborators .` — no matches
- `grep -r resolve_collaborators .` — no matches

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:42Z | groomed | in_design | philippepascal |
| 2026-04-08T15:46Z | in_design | specd | claude-0408-1542-a290 |
| 2026-04-08T21:47Z | specd | ready | apm |
| 2026-04-08T21:47Z | ready | in_progress | philippepascal |