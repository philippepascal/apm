+++
id = "4004f5dc"
title = "Trim state.rs to pure state machine logic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4004f5dc-trim-state-rs-to-pure-state-machine-logi"
created_at = "2026-04-12T06:04:38.471678Z"
updated_at = "2026-04-12T06:37:57.772089Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["4f67992b", "eb4789cf"]
+++

## Spec

### Problem

`state.rs` mixes pure state machine logic (`transition`, `available_transitions`, `append_history`) with unrelated concerns: worktree provisioning (`provision_worktree`), GitHub PR creation (`gh_pr_create_or_update`), git merge/pull operations (`merge_into_default`, `pull_default`), and spec document manipulation (`ensure_amendment_section`).

This ticket trims `state.rs` to only the state machine: `transition()`, `available_transitions()`, and `append_history()`. The extracted functions move to `worktree.rs`, `github.rs`, and `git_util.rs` respectively (all of which exist by the time this ticket is worked, per its dependencies).

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 5 for the full plan.

### Acceptance criteria

- [ ] `gh_pr_create_or_update` is defined as `pub fn` in `github.rs` and absent from `state.rs`
- [ ] `merge_into_default` is defined as `pub fn` in `git_util.rs` and absent from `state.rs`
- [ ] `pull_default` is defined as `pub fn` in `git_util.rs` and absent from `state.rs`
- [ ] `ensure_amendment_section` is defined as `pub fn` in `spec.rs` and absent from `state.rs`
- [ ] `state.rs` exports only `TransitionOutput`, `transition`, `available_transitions`, and `append_history`
- [ ] `transition()` calls `crate::github::gh_pr_create_or_update` in place of the former local private function
- [ ] `transition()` calls `crate::git::merge_into_default` in place of the former local private function
- [ ] `transition()` calls `crate::git::pull_default` in place of the former local private function
- [ ] `transition()` calls `crate::spec::ensure_amendment_section` in place of the former local private function
- [ ] The three PR title tests previously in `state.rs`'s test block are present in `github.rs`'s test block
- [ ] `cargo build --workspace` succeeds with zero errors
- [ ] `cargo test --workspace` passes

### Out of scope

- Moving provision_worktree out of state.rs — done by ticket 4f67992b\n- Renaming git.rs to git_util.rs — done by ticket b28fe914\n- Moving worktree primitives from git.rs to worktree.rs — done by ticket 4f67992b\n- Moving epic branch helpers out of git.rs — done by ticket eb4789cf\n- Any behaviour changes to the moved functions\n- Adding new functionality to any module\n- Updating REFACTOR-CORE.md or other documentation

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:37Z | groomed | in_design | philippepascal |