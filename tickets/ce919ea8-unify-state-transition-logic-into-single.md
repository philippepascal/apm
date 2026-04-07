+++
id = "ce919ea8"
title = "Unify state transition logic into single module"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/ce919ea8-unify-state-transition-logic-into-single"
created_at = "2026-04-07T22:30:50.389099Z"
updated_at = "2026-04-07T22:59:02.485513Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:59Z | groomed | in_design | philippepascal |