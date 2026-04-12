+++
id = "2d0e3534"
title = "Share worktree_for_ticket helper between workers.rs and worktrees.rs"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2d0e3534-share-worktree-for-ticket-helper-between"
created_at = "2026-04-12T09:02:38.703504Z"
updated_at = "2026-04-12T09:20:57.239387Z"
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

- Refactoring workers.rs or worktrees.rs to use CmdContext for config/ticket loading\n- Moving any other helpers into util.rs beyond worktree_for_ticket\n- Adding new functionality to the helper (e.g. creating worktrees that don't exist)\n- Changes to apm-core crates

### Approach

**Create `apm/src/util.rs`**

Add a new file with the following function, taken directly from the existing private function in `workers.rs`:

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

**Register the module in `apm/src/lib.rs`**

Add `pub mod util;` alongside the existing module declarations.

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

1. Create `apm/src/util.rs`
2. Add `pub mod util;` to `apm/src/lib.rs`
3. Update `workers.rs` (remove private fn, add import)
4. Update `worktrees.rs` (replace inline block, clean up imports)
5. `cargo build -p apm` to confirm compilation
6. `cargo test -p apm` to confirm no regressions

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:18Z | groomed | in_design | philippepascal |