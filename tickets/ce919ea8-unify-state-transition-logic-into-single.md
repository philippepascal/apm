+++
id = "ce919ea8"
title = "Unify state transition logic into single module"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "philippepascal"
branch = "ticket/ce919ea8-unify-state-transition-logic-into-single"
created_at = "2026-04-07T22:30:50.389099Z"
updated_at = "2026-04-07T23:02:43.740081Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
depends_on = ["eea2c9bc"]
+++

## Spec

### Problem

State transition logic is scattered across four modules in apm-core. The canonical transition engine lives in `state.rs` (validates target state, updates frontmatter, appends history, executes completion strategy). But `ticket.rs::close()` and `start.rs::run()` bypass it for their specific transitions, duplicating pieces of it inline. Meanwhile `review.rs` owns `available_transitions()`, a function that is conceptually part of the transition system.\n\nThe concrete duplication today is:\n- History-appending logic (12 lines) exists verbatim in both `state.rs::append_history()` and inline inside `ticket.rs::close()`.\n- Worktree provisioning (`git::ensure_worktree` + `git::sync_agent_dirs`) is called as a pair in both the `in_design` branch of `state::transition()` and in `start::run()`, with neither site calling a shared helper.\n- `available_transitions()` filters a state's manually-triggerable transitions; it lives in `review.rs` even though it has nothing to do with the review/edit flow.\n\nA contributor who wants to understand or modify how a transition works must read all four files. The fix is to make `state.rs` the single authoritative module: other modules delegate to it rather than re-implementing pieces.

### Acceptance criteria

- [ ] `ticket::close()` no longer contains inline history-appending code; it calls `state::append_history()` instead
- [ ] `review::available_transitions()` is moved to `state.rs`; `review.rs` re-exports it so existing callers compile unchanged
- [ ] A `provision_worktree` helper exists in `state.rs` (or `git.rs`) and is called by both the `in_design` branch in `state::transition()` and by `start::run()` — neither site contains its own `ensure_worktree` + `sync_agent_dirs` pair
- [ ] `cargo test` passes with no regressions after all changes
- [ ] A contributor reading only `state.rs` can find: history appending, amendment section insertion, available-transition filtering, worktree provisioning, completion strategy execution, and all state-entry validations — without needing to open `ticket.rs` or `review.rs`

### Out of scope

- Routing `start::run()` through `state::transition()` — spawning, worker resolution, and base-branch merging are inherently start-specific and do not belong in the generic transition engine\n- Routing `ticket::close()` entirely through `state::transition()` — the stale-branch lookup (finding a ticket whose branch was already deleted) is close-specific logic that `transition()` does not handle\n- Changing any public type signatures (`TransitionOutput`, `StartOutput`, `CompletionStrategy`, etc.) consumed by the `apm` or `apm-server` crates\n- Adding new transition states, triggers, or completion strategies\n- Moving `review.rs` spec-editing utilities (`split_body`, `apply_review`, `normalize_amendments`) — these are legitimately review-flow concerns\n- Performance or async changes

### Approach

The goal is to make `state.rs` the single authoritative home for transition logic. Three discrete changes accomplish this; they are independent and can be done in any order.

**1. Fix history-appending duplication in `ticket.rs`**

In `ticket::close()` (around line 408), replace the 12-line inline history-append block with a call to the already-public `state::append_history()`:

```rust
// Before (inline in close()):
let row = format!("| {when} | {prev} | closed | {by} |");
if t.body.contains("## History") { ... } else { ... }

// After:
state::append_history(&mut t.body, &prev, "closed", &when, by);
```

Add `use crate::state;` (or adjust the existing import) at the top of `ticket.rs`. No logic change — the two blocks are byte-for-byte identical in behaviour.

**2. Move `available_transitions()` from `review.rs` to `state.rs`**

`available_transitions(config, state) -> Vec<&TransitionConfig>` filters a state's transitions to those with a `command:` trigger and a non-terminal target. It belongs in `state.rs` alongside the rest of the transition machinery.

- Copy the function body into `state.rs` as `pub fn available_transitions(...)`.
- In `review.rs`, replace the function body with a delegation:
  ```rust
  pub fn available_transitions<'a>(config: &'a Config, state: &str) -> Vec<&'a TransitionConfig> {
      crate::state::available_transitions(config, state)
  }
  ```
- No callers outside `review.rs` need changing; the re-export keeps the API stable.

**3. Extract `provision_worktree` helper**

Both `state.rs` (in_design branch, ~lines 175–182) and `start.rs` (~lines 257–259) call:
```rust
let wt = git::ensure_worktree(root, &worktrees_base, &branch)?;
git::sync_agent_dirs(root, &wt, &config.worktrees.agent_dirs);
```

Add to `state.rs`:
```rust
/// Create (or re-use) the worktree for `branch` and sync agent dirs into it.
/// Returns the worktree path.
pub fn provision_worktree(root: &Path, config: &Config, branch: &str) -> Result<PathBuf> {
    let worktrees_base = root.join(&config.worktrees.dir);
    let wt = git::ensure_worktree(root, &worktrees_base, branch)?;
    git::sync_agent_dirs(root, &wt, &config.worktrees.agent_dirs);
    Ok(wt)
}
```

Replace both call sites with `state::provision_worktree(root, config, &branch)?`. The `start.rs` call currently passes `&worktrees_base` that it builds itself — delete that local let-binding once the helper handles it.

**What does NOT change**

- `ticket::close()` continues to own the stale-branch lookup and the merge-to-default logic; only the history-append line is delegated.
- `start::run()` continues to own spawning, worker resolution, and base-branch merging; only the worktree provisioning pair is delegated.
- Public type signatures in `TransitionOutput`, `StartOutput`, `CompletionStrategy`, etc. are untouched.
- No new modules are created; `state.rs` is promoted, not replaced.

**Verification**

After all three changes: `cargo test --workspace` must pass. The tests in `state.rs` (PR title formatting), `start.rs` (profile/worker resolution, 22+ tests), `ticket.rs` (parsing/serialisation), and `review.rs` (split_body, available_transitions, normalize_amendments, apply_review) must all remain green.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:59Z | groomed | in_design | philippepascal |