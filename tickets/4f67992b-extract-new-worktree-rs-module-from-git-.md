+++
id = "4f67992b"
title = "Extract new worktree.rs module from git.rs, state.rs, and ticket.rs"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4f67992b-extract-new-worktree-rs-module-from-git-"
created_at = "2026-04-12T06:04:31.633559Z"
updated_at = "2026-04-12T07:04:51.248151Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["b28fe914"]
+++

## Spec

### Problem

Worktree lifecycle management is currently spread across three unrelated modules. `git_util.rs` (formerly `git.rs`, renamed by ticket b28fe914) owns the low-level primitives: `find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, `sync_agent_dirs`, and their private helpers `clean_agent_dirs`, `is_tracked`, and `copy_dir_recursive`. `state.rs` owns `provision_worktree`, the high-level orchestrator that calls `ensure_worktree` + `sync_agent_dirs`. `ticket.rs` owns `list_worktrees_with_tickets`, which pairs each worktree with its ticket record.\n\nThere is no single place to understand or modify worktree behaviour. A developer who wants to change how worktrees are provisioned must read `state.rs`; one who wants to change how they are discovered must read `git_util.rs`; one who wants to query which tickets have live worktrees must read `ticket.rs`.\n\nThis ticket creates `apm-core/src/worktree.rs` as the single owner of the full worktree lifecycle — discovery, creation, provisioning, agent-directory syncing, and cleanup — and updates all callers to reference it. It runs after b28fe914 (which renames `git.rs` → `git_util.rs` and installs the `pub use git_util as git` compatibility alias).

### Acceptance criteria

- [ ] `apm-core/src/worktree.rs` exists and is declared as `pub mod worktree` in `apm-core/src/lib.rs`
- [ ] `find_worktree_for_branch` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `list_ticket_worktrees` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `ensure_worktree` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `add_worktree` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `remove_worktree` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `sync_agent_dirs` is defined in `worktree.rs` and absent from `git_util.rs`
- [ ] Private helpers `clean_agent_dirs`, `is_tracked`, and `copy_dir_recursive` are defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `provision_worktree` is defined as `pub fn` in `worktree.rs` and absent from `state.rs`
- [ ] `list_worktrees_with_tickets` is defined as `pub fn` in `worktree.rs` and absent from `ticket.rs`
- [ ] Every call-site that previously referenced `git::find_worktree_for_branch`, `git::list_ticket_worktrees`, `git::ensure_worktree`, `git::add_worktree`, `git::remove_worktree`, or `git::sync_agent_dirs` is updated to `worktree::`
- [ ] Every call-site that previously referenced `git::provision_worktree` or `state::provision_worktree` is updated to `worktree::provision_worktree`
- [ ] Every call-site that previously referenced `ticket::list_worktrees_with_tickets` or `git::list_worktrees_with_tickets` is updated to `worktree::list_worktrees_with_tickets`
- [ ] `cargo build` succeeds with zero errors across `apm-core`, `apm`, and `apm-server`
- [ ] `cargo test` passes (integration suite included)

### Out of scope

- Renaming `git.rs` to `git_util.rs` — done by ticket b28fe914, which this ticket depends on
- Moving `merge_into_default` and `pull_default` from `state.rs` — also handled by b28fe914
- Creating `ticket_fmt.rs` or `epic.rs` — separate tickets in the same epic
- Behaviour changes to any moved function — this is a pure code relocation
- Changing public API signatures or return types
- Updating `REFACTOR-CORE.md` or other documentation
- Adding new worktree functionality beyond what already exists

### Approach

All changes are in `apm-core/` unless noted. Start from the state left by b28fe914: `git.rs` is already `git_util.rs` and `lib.rs` already has `pub use git_util as git`.

**1. Create `apm-core/src/worktree.rs`**

Add the following imports at the top (mirroring what the functions need):
```rust
use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::logger;
use crate::git_util::run;
```

`run` is `pub(crate)` in `git_util.rs` after b28fe914 — import it directly rather than duplicating it.

**2. Move from `git_util.rs` into `worktree.rs`**

Cut these nine items verbatim — do not change signatures or logic:
- `pub fn find_worktree_for_branch(root: &Path, branch: &str) -> Option<PathBuf>`
- `pub fn list_ticket_worktrees(root: &Path) -> Result<Vec<(PathBuf, String)>>`
- `pub fn ensure_worktree(root: &Path, worktrees_base: &Path, branch: &str) -> Result<PathBuf>`
- `pub fn add_worktree(root: &Path, wt_path: &Path, branch: &str) -> Result<()>`
- `pub fn remove_worktree(root: &Path, wt_path: &Path, force: bool) -> Result<()>`
- `pub fn sync_agent_dirs(root: &Path, wt_path: &Path, agent_dirs: &[String], warnings: &mut Vec<String>)`
- `fn clean_agent_dirs(root: &Path, wt_path: &Path)` (private)
- `fn is_tracked(root: &Path, path: &str) -> bool` (private)
- `fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()>` (private)

All of these call `run()`; the import from step 1 covers that dependency. Remove the original definitions from `git_util.rs`.

**3. Move `provision_worktree` from `state.rs` into `worktree.rs`**

The real signature (verify against `state.rs` before cutting):
```rust
pub fn provision_worktree(
    root: &Path,
    config: &Config,
    branch: &str,
    warnings: &mut Vec<String>,
) -> Result<PathBuf>
```

Cut the function body into `worktree.rs`. Internal calls to `ensure_worktree` and `sync_agent_dirs` need no module prefix within the same file. The `Config` import is already present from step 1.

In `state.rs`, remove the function definition. Check callers with:
```
grep -r provision_worktree apm-core/ apm/ apm-server/
```
Update each call-site to `worktree::provision_worktree(...)`.

**4. Move `list_worktrees_with_tickets` from `ticket.rs` into `worktree.rs`**

Current signature:
```rust
pub fn list_worktrees_with_tickets(root: &Path) -> Result<Vec<(PathBuf, Ticket)>>
```

This function references `Ticket` and calls `load_all_from_git`. After moving to `worktree.rs`, add:
```rust
use crate::ticket::{Ticket, load_all_from_git};
```

Remove the function from `ticket.rs`. Confirm no circular import: `worktree.rs` → `ticket.rs` is fine so long as `ticket.rs` does not import from `worktree`. Verify with a search before committing.

**5. Update `apm-core/src/lib.rs`**

Add `pub mod worktree;`. If b28fe914 added a stub declaration, replace it with the real one. The existing `pub use git_util as git` alias does not cover `worktree` — callers must use `worktree::` directly.

**6. Update all call-sites**

Search across `apm-core/`, `apm/`, and `apm-server/`:
- `git::find_worktree_for_branch` → `worktree::find_worktree_for_branch`
- `git::list_ticket_worktrees` → `worktree::list_ticket_worktrees`
- `git::ensure_worktree` → `worktree::ensure_worktree`
- `git::add_worktree` → `worktree::add_worktree`
- `git::remove_worktree` → `worktree::remove_worktree`
- `git::sync_agent_dirs` → `worktree::sync_agent_dirs`
- `state::provision_worktree` → `worktree::provision_worktree`
- `ticket::list_worktrees_with_tickets` → `worktree::list_worktrees_with_tickets`

Add `use apm_core::worktree;` (or `use crate::worktree;` inside `apm-core`) to each file that gains new worktree references.

**7. Verify**

Run `cargo build` then `cargo test` from the repo root. Fix any compilation errors (typically missing imports or stray references). No logic changes are permitted during fixes.

### Open questions


### Amendment requests

- [x] Remove the duplicated `run()` helper from the Approach. Instead, import `crate::git_util::run()` — ticket b28fe914 will make `run()` `pub(crate)` in `git_util.rs`. Do not duplicate the git invocation wrapper.
- [x] Fix `provision_worktree` signature in the Approach — the spec shows `(root, worktrees_base, branch, agent_dirs, warnings)` but the actual signature is `pub fn provision_worktree(root: &Path, config: &Config, branch: &str, warnings: &mut Vec<String>) -> Result<PathBuf>`. Use the real signature.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:27Z | groomed | in_design | philippepascal |
| 2026-04-12T06:32Z | in_design | specd | claude-0412-0627-92a0 |
| 2026-04-12T06:54Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T07:04Z | ammend | in_design | philippepascal |