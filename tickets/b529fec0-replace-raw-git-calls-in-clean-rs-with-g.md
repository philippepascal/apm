+++
id = "b529fec0"
title = "Replace raw git calls in clean.rs with git_util helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b529fec0-replace-raw-git-calls-in-clean-rs-with-g"
created_at = "2026-04-12T17:29:27.980274Z"
updated_at = "2026-04-12T17:36:47.963110Z"
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

- [ ] clean.rs contains no `Command::new("git")` calls
- [ ] clean.rs has no `use std::process::Command` import
- [ ] `diagnose_worktree` produces identical categorisation output to before (same three-bucket logic, same error propagation via `?`)
- [ ] The `wt_clean` check in `candidates` uses `git_util::is_worktree_dirty()`
- [ ] Both local-branch-exists checks in `candidates` use `git_util::local_branch_exists()`
- [ ] The branch deletion in `remove` uses `git_util::delete_local_branch()`
- [ ] The remote-tracking prune in `remove` uses `git_util::prune_remote_tracking()`
- [ ] `cargo build` succeeds with no new warnings
- [ ] All existing tests pass unchanged

### Out of scope

- Adding new git_util helpers (covered by ticket 061d0ac1)
- Changing the behaviour or public API of any clean.rs function
- Refactoring the `DirtyWorktree` struct or the categorisation logic inside `diagnose_worktree`
- Raw git calls in any file other than clean.rs

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
| 2026-04-12T17:36Z | groomed | in_design | philippepascal |