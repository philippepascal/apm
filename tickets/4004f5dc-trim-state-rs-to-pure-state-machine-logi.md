+++
id = "4004f5dc"
title = "Trim state.rs to pure state machine logic"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4004f5dc-trim-state-rs-to-pure-state-machine-logi"
created_at = "2026-04-12T06:04:38.471678Z"
updated_at = "2026-04-12T07:58:18.035363Z"
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

- [x] `gh_pr_create_or_update` is defined as `pub fn` in `github.rs` and absent from `state.rs`
- [x] `transition()` calls `crate::github::gh_pr_create_or_update` in place of the former local private function
- [x] The three PR title tests previously in `state.rs`'s test block are present in `github.rs`'s test block
- [x] `state.rs` imports that were only needed by `gh_pr_create_or_update` are removed
- [x] `cargo build --workspace` succeeds with zero errors
- [x] `cargo test --workspace` passes

### Out of scope

- Moving provision_worktree out of state.rs — done by ticket 4f67992b
- Renaming git.rs to git_util.rs — done by ticket b28fe914
- Moving worktree primitives from git.rs to worktree.rs — done by ticket 4f67992b
- Moving epic branch helpers out of git.rs — done by ticket eb4789cf
- Moving merge_into_default and pull_default — done by ticket b28fe914
- Moving ensure_amendment_section — owned by ticket a6367b87
- Any behaviour changes to the moved functions
- Adding new functionality to any module
- Updating REFACTOR-CORE.md or other documentation

### Approach

The starting point is `state.rs` after tickets b28fe914, 4f67992b, and eb4789cf have all landed. At that point `provision_worktree`, `merge_into_default`, and `pull_default` are already absent from `state.rs`, and `transition()` already calls them via their new module paths. The only remaining non-state-machine function this ticket targets is `gh_pr_create_or_update`. (`ensure_amendment_section` remains in `state.rs` for ticket a6367b87 to handle.)

**1. Move `gh_pr_create_or_update` to `github.rs`**

- Cut the function verbatim from `state.rs`; paste into `apm-core/src/github.rs` as `pub fn`.
- `github.rs` already uses `std::process::Command` and `anyhow` — add any missing imports.
- Extract the inline PR-title logic into a private helper in `github.rs`:
  `fn pr_title(id: &str, title: &str) -> String` — `short_id` is `&id[..8.min(id.len())]`, returns `short_id` alone when `title` is empty, else `"short_id: title"`. This matches the test helper that already lives in `state.rs`'s test block.
- Update `gh_pr_create_or_update` to call `pr_title(id, title)`.
- Move the three `pr_title_*` tests from `state.rs`'s `#[cfg(test)]` block to `github.rs`'s `#[cfg(test)]` block unchanged. They reference the now-local `pr_title` helper.

**2. Update `transition()` in `state.rs`**

Replace the private-function call:
- `gh_pr_create_or_update(...)` → `crate::github::gh_pr_create_or_update(...)`

**3. Clean up `state.rs` imports**

Remove any `use` items that were only needed by `gh_pr_create_or_update`. Verify remaining imports (e.g. `use crate::{config::{CompletionStrategy, Config}, git, ticket};`) are still required by `transition()`, `available_transitions()`, and `append_history()`.

**4. Verify**

Run `cargo build --workspace` and fix any compilation errors (missing visibility, stale imports). Then run `cargo test --workspace`. No logic changes are permitted during fixes.

### Open questions


### Amendment requests

- [x] Remove `ensure_amendment_section` from this ticket's scope entirely. Ticket a6367b87 (which depends on this one) owns moving it from `state.rs` to `review.rs`. Do not move it to `spec.rs` or anywhere else — leave it in `state.rs` for a6367b87 to handle.
- [x] Remove `merge_into_default` and `pull_default` from this ticket's scope. Ticket b28fe914 owns absorbing these into `git_util.rs`. By the time this ticket is worked (it depends on 4f67992b and eb4789cf, which depend on b28fe914), these functions will already be in `git_util.rs`.
- [x] Update acceptance criteria to reflect the reduced scope: this ticket only moves `gh_pr_create_or_update` from `state.rs` to `github.rs` and updates `transition()` to call the moved functions via their new module paths. State.rs should also be cleaned of any imports that were only needed by the already-moved functions.
- [x] Fix `\n` formatting in Out of scope section — literal backslash-n characters appear instead of real newlines.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:37Z | groomed | in_design | philippepascal |
| 2026-04-12T06:44Z | in_design | specd | claude-0412-0638-0d80 |
| 2026-04-12T06:54Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T07:02Z | ammend | in_design | philippepascal |
| 2026-04-12T07:04Z | in_design | specd | claude-0412-0702-6e90 |
| 2026-04-12T07:13Z | specd | ready | apm |
| 2026-04-12T07:58Z | ready | in_progress | philippepascal |