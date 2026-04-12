+++
id = "4004f5dc"
title = "Trim state.rs to pure state machine logic"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4004f5dc-trim-state-rs-to-pure-state-machine-logi"
created_at = "2026-04-12T06:04:38.471678Z"
updated_at = "2026-04-12T06:44:39.881377Z"
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

The starting point is `state.rs` after ticket 4f67992b has landed: `provision_worktree` is already absent and `transition()` already calls `worktree::provision_worktree`. The four remaining non-state-machine functions are `gh_pr_create_or_update`, `merge_into_default`, `pull_default`, and `ensure_amendment_section`.

**1. Move `gh_pr_create_or_update` to `github.rs`**

- Cut the function verbatim from `state.rs`; paste into `apm-core/src/github.rs` as `pub fn`.
- `github.rs` already uses `std::process::Command` and `anyhow` — add any missing imports.
- Extract the inline PR-title logic into a private helper in `github.rs`:
  `fn pr_title(id: &str, title: &str) -> String` — short_id is `&id[..8.min(id.len())]`, returns the short_id alone when title is empty, else `"short_id: title"`. This matches the test helper that already lives in `state.rs`'s test block.
- Update `gh_pr_create_or_update` to call `pr_title(id, title)`.
- Move the three `pr_title_*` tests from `state.rs`'s `#[cfg(test)]` block to `github.rs`'s `#[cfg(test)]` block unchanged. They reference the now-local `pr_title` helper.

**2. Move `merge_into_default` and `pull_default` to `git_util.rs`**

- Cut both functions from `state.rs` and paste into `apm-core/src/git_util.rs` (the b28fe914-renamed `git.rs`) as `pub fn`.
- Update internal references inside the moved functions:
  - `merge_into_default`: `git::ensure_worktree(...)` becomes `crate::worktree::ensure_worktree(...)`; `git::push_branch(...)` becomes the local `push_branch(...)` (already in the same file).
  - `pull_default`: `git::find_worktree_for_branch(...)` becomes `crate::worktree::find_worktree_for_branch(...)`.
- `merge_into_default` takes `config: &Config` — verify `use crate::config::Config` is already in `git_util.rs`; add if missing.
- No circular dependency: `worktree.rs` uses only its own local `run()` helper and does not import `git_util`, so `git_util -> worktree` is a one-way edge.

**3. Move `ensure_amendment_section` to `spec.rs`**

- Cut the function from `state.rs` and paste into `apm-core/src/spec.rs` as `pub fn`. No new imports needed — the function is pure `String` manipulation.

**4. Update `transition()` in `state.rs`**

Replace the four private-function calls with module-qualified equivalents:
- `ensure_amendment_section(...)` -> `crate::spec::ensure_amendment_section(...)`
- `gh_pr_create_or_update(...)` -> `crate::github::gh_pr_create_or_update(...)`
- `merge_into_default(...)` -> `crate::git::merge_into_default(...)` (using the `pub use git_util as git` alias from b28fe914)
- `pull_default(...)` -> `crate::git::pull_default(...)`

**5. Clean up `state.rs` imports**

Remove any `use` items that were only needed by the moved functions. Verify the existing `use crate::{config::{CompletionStrategy, Config}, git, ticket};` line remains intact.

**6. Verify**

Run `cargo build --workspace` and fix any compilation errors (missing visibility, stale imports). Then run `cargo test --workspace`. No logic changes are permitted during fixes.

### Open questions


### Amendment requests

- [ ] Remove `ensure_amendment_section` from this ticket's scope entirely. Ticket a6367b87 (which depends on this one) owns moving it from `state.rs` to `review.rs`. Do not move it to `spec.rs` or anywhere else — leave it in `state.rs` for a6367b87 to handle.
- [ ] Remove `merge_into_default` and `pull_default` from this ticket's scope. Ticket b28fe914 owns absorbing these into `git_util.rs`. By the time this ticket is worked (it depends on 4f67992b and eb4789cf, which depend on b28fe914), these functions will already be in `git_util.rs`.
- [ ] Update acceptance criteria to reflect the reduced scope: this ticket only moves `gh_pr_create_or_update` from `state.rs` to `github.rs` and updates `transition()` to call the moved functions via their new module paths. State.rs should also be cleaned of any imports that were only needed by the already-moved functions.
- [ ] Fix `\n` formatting in Out of scope section — literal backslash-n characters appear instead of real newlines.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:37Z | groomed | in_design | philippepascal |
| 2026-04-12T06:44Z | in_design | specd | claude-0412-0638-0d80 |