+++
id = "2d0e3534"
title = "Share worktree_for_ticket helper between workers.rs and worktrees.rs"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2d0e3534-share-worktree-for-ticket-helper-between"
created_at = "2026-04-12T09:02:38.703504Z"
updated_at = "2026-04-12T10:36:21.952965Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f"]
+++

## Spec

### Problem

`apm/src/cmd/workers.rs` defines a private helper `worktree_for_ticket(root, id_arg)` (lines ~196–213) that resolves a ticket ID argument to its worktree path and canonical ID. It loads config and tickets from git, resolves the ID, derives the branch name, and calls `worktree::find_worktree_for_branch`. The function is used by both `tail_log()` and `kill()` in that file.

`apm/src/cmd/worktrees.rs::remove()` (lines ~40–53) contains the same logic inlined: it loads tickets, resolves the ID, derives the branch name, and finds the worktree — all before calling `worktree::remove_worktree`. The two blocks are functionally identical (same crate imports, same call sequence, same fallback branch-name logic).

Because the shared helper lives as a private function in `workers.rs`, `worktrees.rs` cannot call it and must duplicate it. The fix is to lift the function into `apm/src/util.rs` (a new module) so both command files can import it from a single source of truth.

### Acceptance criteria

- [x] `apm/src/util.rs` exists and is declared as `pub mod util;` in `apm/src/lib.rs`
- [x] `util::worktree_for_ticket(root, id_arg)` compiles and returns `Result<(PathBuf, String)>`
- [x] `apm/src/cmd/workers.rs` no longer defines its own `worktree_for_ticket` function
- [x] `apm/src/cmd/workers.rs` calls `crate::util::worktree_for_ticket` for both `tail_log` and `kill`
- [x] `apm/src/cmd/worktrees.rs::remove()` no longer contains the inline ticket-to-worktree resolution block
- [x] `apm/src/cmd/worktrees.rs::remove()` calls `crate::util::worktree_for_ticket` to obtain the worktree path
- [x] `cargo build -p apm` succeeds with no new warnings
- [x] `cargo test -p apm` passes (all existing tests continue to pass)

### Out of scope

- Refactoring workers.rs or worktrees.rs to use CmdContext for config/ticket loading\n- Moving any other helpers into util.rs beyond worktree_for_ticket\n- Adding new functionality to the helper (e.g. creating worktrees that don't exist)\n- Changes to apm-core crates

### Approach

**Prerequisites**

This ticket depends on d3ebdc0f, which creates `apm/src/util.rs` and registers it with `pub mod util;` in `apm/src/lib.rs`. When this ticket starts, both of those already exist. The `worktree_for_ticket` function is appended to the existing `util.rs`; there is no need to create the file or touch `lib.rs`.

**Add `worktree_for_ticket` to existing `apm/src/util.rs`**

Append the following function to `apm/src/util.rs` (the file was created by d3ebdc0f and already has at least one item in it):

```rust
use anyhow::Result;
use apm_core::{config::Config, ticket, ticket_fmt, worktree};
use std::path::{Path, PathBuf};

/// Resolve a ticket ID argument to its worktree path and canonical ticket ID.
/// Loads config and tickets from git internally.
pub fn worktree_for_ticket(root: &Path, id_arg: &str) -> Result<(PathBuf, String)> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;
    let t = tickets
        .iter()
        .find(|t| t.frontmatter.id == id)
        .ok_or_else(|| anyhow::anyhow!("ticket {id:?} not found"))?;
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt = worktree::find_worktree_for_branch(root, &branch)
        .ok_or_else(|| anyhow::anyhow!("no worktree for ticket {id:?}"))?;
    Ok((wt, id))
}
```

Consolidate any `use` statements with what d3ebdc0f already added at the top of the file to avoid duplicate imports.

**Update `apm/src/cmd/workers.rs`**

- Remove the private `fn worktree_for_ticket(...)` definition (lines ~196–213).
- Add `use crate::util::worktree_for_ticket;` to the imports at the top of the file (or call it as `crate::util::worktree_for_ticket` at the call sites — either is fine).
- No changes needed to `tail_log()` or `kill()` call sites; the function signature is unchanged.

**Update `apm/src/cmd/worktrees.rs`**

- In `remove(root, config, id_arg)`, replace the inline block that loads tickets, resolves the ID, derives the branch, and calls `find_worktree_for_branch` with:
  ```rust
  let (wt_path, _id) = crate::util::worktree_for_ticket(root, id_arg)?;
  ```
- The `config` parameter may still be needed by other parts of `remove()` (e.g., for the tickets dir), but the inline ticket-loading block is replaced entirely.
- Remove any imports from `worktrees.rs` that are now only used by the deleted block (`ticket`, `ticket_fmt` imports if no longer referenced elsewhere in the file).

**Order of changes**

1. Append `worktree_for_ticket` to existing `apm/src/util.rs` (no new file, no lib.rs change)
2. Update `workers.rs` (remove private fn, add import)
3. Update `worktrees.rs` (replace inline block, clean up imports)
4. `cargo build -p apm` to confirm compilation
5. `cargo test -p apm` to confirm no regressions

### Open questions


### Amendment requests

- [x] Add d3ebdc0f as a dependency — both tickets independently create `apm/src/util.rs` and add `pub mod util;` to `lib.rs`. Without this dependency, whichever lands second will hit a merge conflict or silently overwrite the first ticket's content.
- [x] Update the spec to note that `util.rs` already exists (created by d3ebdc0f) when this ticket starts. The `worktree_for_ticket` function should be appended to the existing file, not written as a standalone new file. Remove the "Create apm/src/util.rs" step and replace with "Add to existing apm/src/util.rs". Remove the "Add pub mod util; to lib.rs" step (already done by d3ebdc0f).

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:18Z | groomed | in_design | philippepascal |
| 2026-04-12T09:21Z | in_design | specd | claude-0412-0918-aab0 |
| 2026-04-12T10:11Z | specd | ammend | apm |
| 2026-04-12T10:12Z | ammend | in_design | philippepascal |
| 2026-04-12T10:13Z | in_design | specd | claude-0412-1012-5b90 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T10:36Z | ready | in_progress | philippepascal |