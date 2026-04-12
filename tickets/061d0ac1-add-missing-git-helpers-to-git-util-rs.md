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

- Updating callers (clean.rs, epic.rs, init.rs, start.rs, worktree.rs) to use the new helpers — covered by subsequent tickets in the same epic
- Removing or replacing existing git_util helpers that are already present
- Adding worktree management helpers (add/remove worktree) — worktree.rs already uses `run()` for these
- Adding push or fetch helpers beyond what already exists
- Handling the `diagnose_worktree()` deep status parsing in clean.rs (categorising known_temp / other_untracked / modified_tracked) — that logic belongs in clean.rs
- Test helpers (git_init, git_cmd) in test modules — those are intentionally local

### Approach

All changes are additions to `apm-core/src/git_util.rs`. No existing functions are modified. Each new function is `pub fn` and follows the existing convention: use `run()` where a `Result<String>` return is acceptable; use `Command::new("git")` directly (via `std::process::Command`) only when the caller needs bool/silent/non-fatal semantics that `run()` does not support.

**1. `pub fn is_worktree_dirty(path: &Path) -> bool`**
Run `git -C <path> status --porcelain`. Return `true` if stdout is non-empty. Use `Command::new("git").args(["-C", ..., "status", "--porcelain"]).output()`; treat any error as clean (return `false`).

**2. `pub fn local_branch_exists(root: &Path, branch: &str) -> bool`**
Run `git -C <root> rev-parse --verify refs/heads/<branch>`. Return `output.status.success()`. Treat command failure as `false`.

**3. `pub fn delete_local_branch(root: &Path, branch: &str, warnings: &mut Vec<String>)`**
Run `git -C <root> branch -D <branch>`. On non-zero exit, push `format!("warning: could not delete branch {branch}: {stderr}")` to warnings. Never return `Err`.

**4. `pub fn prune_remote_tracking(root: &Path, branch: &str)`**
Run `git -C <root> branch -dr origin/<branch>`. Ignore all errors (use `let _ = ...`).

**5. `pub fn stage_files(root: &Path, files: &[&str]) -> Result<()>`**
Build args `["add"] ++ files` and call `run(root, &args)`. Discard the stdout string, return `Ok(())` on success. The `run()` helper already formats stderr into the error on failure.

**6. `pub fn commit(root: &Path, message: &str) -> Result<()>`**
Call `run(root, &["commit", "-m", message])`. Discard stdout, return `Ok(())`.

**7. `pub fn git_config_get(root: &Path, key: &str) -> Option<String>`**
Run `git -C <root> config <key>` via `Command::new("git")`. On success, return `Some(stdout.trim().to_string())`, filtering empty strings. Return `None` on non-zero exit or command error. (Cannot use `run()` because a missing key exits non-zero and should not be an error.)

**8. `pub fn merge_ref(root: &Path, refname: &str, warnings: &mut Vec<String>) -> Option<String>`**
- First verify the ref exists: `git -C <root> rev-parse --verify <refname>`. If that fails, fall back — but the caller in start.rs already picks between `remote_ref` and `merge_base`, so this helper receives only a pre-validated refname. Simply run `git -C <wt_path> merge <refname> --no-edit`.
- On success: if stdout contains "Already up to date", return `None`. Otherwise return `Some(format!("Merged {refname} into branch."))`.
- On non-zero exit: push `format!("warning: merge {refname} failed: {stderr}")` to warnings, return `None`.
- On command error: push a warning, return `None`.
- Note: the caller (start.rs) currently passes a `wt_display` path as the working directory, not `root`. The helper signature therefore takes `dir: &Path` (not `root`) to match this usage. Name it accordingly: `pub fn merge_ref(dir: &Path, refname: &str, warnings: &mut Vec<String>) -> Option<String>`.

**9. `pub fn is_file_tracked(root: &Path, path: &str) -> bool`**
Run `git ls-files --error-unmatch <path>` with `current_dir(root)`, suppressing stdout and stderr. Return `status.success()`. Treat command error as `false`.

**Placement**: append all nine functions after the existing `pull_default` function at the bottom of `git_util.rs`, before the `#[cfg(test)]` block if one exists.

**No new dependencies** are required; all git invocations use `std::process::Command` which is already imported.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:31Z | groomed | in_design | philippepascal |