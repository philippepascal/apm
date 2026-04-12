+++
id = "2d0e3534"
title = "Share worktree_for_ticket helper between workers.rs and worktrees.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2d0e3534-share-worktree-for-ticket-helper-between"
created_at = "2026-04-12T09:02:38.703504Z"
updated_at = "2026-04-12T09:18:27.704226Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

`apm/src/cmd/workers.rs` defines a private helper `worktree_for_ticket(root, id_arg)` (lines ~196–213) that resolves a ticket ID argument to its worktree path and canonical ID. It loads config and tickets from git, resolves the ID, derives the branch name, and calls `worktree::find_worktree_for_branch`. The function is used by both `tail_log()` and `kill()` in that file.

`apm/src/cmd/worktrees.rs::remove()` (lines ~40–53) contains the same logic inlined: it loads tickets, resolves the ID, derives the branch name, and finds the worktree — all before calling `worktree::remove_worktree`. The two blocks are functionally identical (same crate imports, same call sequence, same fallback branch-name logic).

Because the shared helper lives as a private function in `workers.rs`, `worktrees.rs` cannot call it and must duplicate it. The fix is to lift the function into `apm/src/util.rs` (a new module) so both command files can import it from a single source of truth.

### Acceptance criteria

- [ ] `apm/src/util.rs` exists and is declared as `pub mod util;` in `apm/src/lib.rs`
- [ ] `util::worktree_for_ticket(root, id_arg)` compiles and returns `Result<(PathBuf, String)>`
- [ ] `apm/src/cmd/workers.rs` no longer defines its own `worktree_for_ticket` function
- [ ] `apm/src/cmd/workers.rs` calls `crate::util::worktree_for_ticket` for both `tail_log` and `kill`
- [ ] `apm/src/cmd/worktrees.rs::remove()` no longer contains the inline ticket-to-worktree resolution block
- [ ] `apm/src/cmd/worktrees.rs::remove()` calls `crate::util::worktree_for_ticket` to obtain the worktree path
- [ ] `cargo build -p apm` succeeds with no new warnings
- [ ] `cargo test -p apm` passes (all existing tests continue to pass)

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:18Z | groomed | in_design | philippepascal |