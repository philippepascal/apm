+++
id = "b529fec0"
title = "Replace raw git calls in clean.rs with git_util helpers"
state = "implemented"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b529fec0-replace-raw-git-calls-in-clean-rs-with-g"
created_at = "2026-04-12T17:29:27.980274Z"
updated_at = "2026-04-12T18:03:32.084130Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
depends_on = ["061d0ac1"]
+++

## Spec

### Problem

clean.rs contains 6 raw `Command::new("git")` calls, making it the last file in apm-core that bypasses the `git_util` abstraction layer. The goal is zero raw git invocations in clean.rs — all git interaction goes through git_util so error handling, path quoting, and command construction are consistent across the codebase.

Five of the six calls map directly to helpers that ticket 061d0ac1 adds to git_util: `is_worktree_dirty`, `local_branch_exists`, `delete_local_branch`, and `prune_remote_tracking`. The sixth call — in `diagnose_worktree` — runs `git status --porcelain` and parses the full output line-by-line to categorise files into three buckets (known temp files, other untracked, modified tracked). It cannot be replaced by `is_worktree_dirty()` (which only returns a bool), but it can use the crate-internal `git_util::run()` helper, which handles spawning, exit-code checking, and stdout decoding. Since clean.rs and git_util.rs are in the same crate (`apm-core`), `pub(crate) fn run()` is accessible.

This ticket must land after 061d0ac1 is merged into the epic branch, because it consumes the helpers that ticket adds.

### Acceptance criteria

- [x] clean.rs contains no `Command::new("git")` calls
- [x] clean.rs has no `use std::process::Command` import
- [x] `diagnose_worktree` produces identical categorisation output to before (same three-bucket logic, same error propagation via `?`)
- [x] The `wt_clean` check in `candidates` uses `git_util::is_worktree_dirty()`
- [x] Both local-branch-exists checks in `candidates` use `git_util::local_branch_exists()`
- [x] The branch deletion in `remove` uses `git_util::delete_local_branch()`
- [x] The remote-tracking prune in `remove` uses `git_util::prune_remote_tracking()`
- [x] `cargo build` succeeds with no new warnings
- [x] All existing tests pass unchanged

### Out of scope

- Adding new git_util helpers (covered by ticket 061d0ac1)
- Changing the behaviour or public API of any clean.rs function
- Refactoring the `DirtyWorktree` struct or the categorisation logic inside `diagnose_worktree`
- Raw git calls in any file other than clean.rs

### Approach

File to change: `apm-core/src/clean.rs`

**Prerequisites:** ticket 061d0ac1 must be merged into the epic branch first.

**Call 1 - diagnose_worktree (~line 49)**

Replace the Command::new("git") .args(["-C", path, "status", "--porcelain"]).output()? and the String::from_utf8_lossy line with:
  let stdout = git_util::run(path, &["status", "--porcelain"])?;

run() returns Result<String> (trimmed stdout). The downstream for-loop over stdout.lines() is unchanged.

**Call 2 - candidates wt_clean check (~line 156)**

Replace the Command + match block that sets wt_clean with:
  let wt_clean = !git_util::is_worktree_dirty(&path);

is_worktree_dirty returns false on error (treats errors as dirty), matching current behaviour.

**Calls 3 and 4 - candidates local-branch-exists (~lines 181, 234)**

Replace each Command block with:
  let lbe = git_util::local_branch_exists(&root, &branch);
  // and:
  let local_branch_exists = git_util::local_branch_exists(&root, &branch);

Both helpers return false on error, matching current behaviour.

**Call 5 - remove branch deletion (~line 272)**

Replace the Command::new("git").args([..., "branch", "-D", ...]) block and its match with:
  git_util::delete_local_branch(&root, &candidate.branch, &mut warnings);

The helper pushes a warning into warnings on failure. Any success-path log messages outside the raw command call stay in place.

**Call 6 - remove remote-tracking prune (~line 300)**

Replace the let _ = Command::new("git") .args([..., "branch", "-dr", ...]).output() call with:
  git_util::prune_remote_tracking(&root, &candidate.branch);

Errors are silently ignored by the helper, matching the current let _ = behaviour.

**Cleanup**

Remove `use std::process::Command;` from clean.rs imports. Verify with cargo build and cargo test.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:36Z | groomed | in_design | philippepascal |
| 2026-04-12T17:41Z | in_design | specd | claude-0412-1736-a938 |
| 2026-04-12T17:54Z | specd | ready | apm |
| 2026-04-12T17:59Z | ready | in_progress | philippepascal |
| 2026-04-12T18:03Z | in_progress | implemented | claude-0412-1800-2528 |
