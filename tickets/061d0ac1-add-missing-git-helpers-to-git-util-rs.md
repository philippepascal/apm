+++
id = "061d0ac1"
title = "Add missing git helpers to git_util.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/061d0ac1-add-missing-git-helpers-to-git-util-rs"
created_at = "2026-04-12T17:29:22.472769Z"
updated_at = "2026-04-12T17:31:57.684129Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
+++

## Spec

### Problem

21 raw `Command::new("git")` calls are scattered across `clean.rs`, `epic.rs`, `init.rs`, `start.rs`, and `worktree.rs`, bypassing `git_util.rs` entirely. These modules construct git commands directly, duplicating argument patterns and spreading git implementation details throughout the codebase.

`git_util.rs` already defines a `run()` helper that centralises command construction, error formatting, and stdout capture — but nine behaviours are absent from its public API:

- detecting whether a worktree has uncommitted changes
- checking whether a local branch ref exists
- deleting a local branch (non-fatal)
- pruning a remote-tracking ref (silent)
- staging a list of files
- creating a commit from the working tree
- reading a git config key
- merging an arbitrary ref with output reporting
- checking whether a path is tracked by git

Because these helpers are missing, callers must either inline the git command or (in the case of `has_commits` and `fetch_branch`) re-implement helpers that already exist in `git_util.rs`.

This ticket adds the nine missing helpers. A separate ticket will update each caller to use them.

### Acceptance criteria

- [ ] `git_util::is_worktree_dirty(path: &Path) -> bool` returns `true` when `git status --porcelain` produces any output for the given path
- [ ] `git_util::is_worktree_dirty` returns `false` when the working tree is clean
- [ ] `git_util::local_branch_exists(root: &Path, branch: &str) -> bool` returns `true` when `refs/heads/<branch>` resolves
- [ ] `git_util::local_branch_exists` returns `false` when the branch does not exist locally
- [ ] `git_util::delete_local_branch(root: &Path, branch: &str, warnings: &mut Vec<String>)` deletes the branch and does not push to warnings on success
- [ ] `git_util::delete_local_branch` pushes a warning message (not a hard error) when deletion fails
- [ ] `git_util::prune_remote_tracking(root: &Path, branch: &str)` runs `git branch -dr origin/<branch>` and silently ignores any failure
- [ ] `git_util::stage_files(root: &Path, files: &[&str]) -> Result<()>` stages exactly the listed paths and returns `Ok(())` on success
- [ ] `git_util::stage_files` returns an error when `git add` fails (e.g. path does not exist)
- [ ] `git_util::commit(root: &Path, message: &str) -> Result<()>` creates a commit with the given message and returns `Ok(())` on success
- [ ] `git_util::commit` returns an error when `git commit` fails (e.g. nothing staged)
- [ ] `git_util::git_config_get(root: &Path, key: &str) -> Option<String>` returns `Some(value)` trimmed of whitespace when the key exists
- [ ] `git_util::git_config_get` returns `None` when the key is absent or git exits non-zero
- [ ] `git_util::merge_ref(root: &Path, refname: &str, warnings: &mut Vec<String>) -> Option<String>` returns `Some(message)` describing the merge when the ref exists and the merge succeeds
- [ ] `git_util::merge_ref` returns `None` and pushes a warning when the merge fails
- [ ] `git_util::merge_ref` returns `None` without a warning when the result is already up to date
- [ ] `git_util::is_file_tracked(root: &Path, path: &str) -> bool` returns `true` when `git ls-files --error-unmatch` exits zero for the given path
- [ ] `git_util::is_file_tracked` returns `false` when the path is not tracked
- [ ] All nine functions are exported as `pub fn` from `git_util.rs`

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
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:31Z | groomed | in_design | philippepascal |