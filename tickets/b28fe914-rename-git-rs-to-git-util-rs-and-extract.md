+++
id = "b28fe914"
title = "Rename git.rs to git_util.rs and extract non-git functions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b28fe914-rename-git-rs-to-git-util-rs-and-extract"
created_at = "2026-04-12T06:04:25.779848Z"
updated_at = "2026-04-12T06:21:50.078736Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["4660b156"]
+++

## Spec

### Problem

`git.rs` has grown into a catch-all module. It contains genuine git plumbing (branch, commit, push, merge operations) alongside unrelated concerns: worktree lifecycle management, epic branch helpers, ticket ID generation (`gen_hex_id`), and ticket branch name parsing (`resolve_ticket_branch`, `branch_name_from_path`).

This ticket renames `git.rs` → `git_util.rs` and moves non-git functions to their proper homes: ticket format helpers to `ticket_fmt.rs` (created by 4660b156), worktree functions to `worktree.rs` (created by 4f67992b), and epic helpers to `epic.rs` (handled by eb4789cf). It also absorbs `merge_into_default()` and `pull_default()` from `state.rs` since those are git operations.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 2 for the full plan.

### Acceptance criteria

- [ ] `apm-core/src/git.rs` no longer exists; `apm-core/src/git_util.rs` exists in its place containing only genuine git plumbing
- [ ] `apm-core/src/lib.rs` declares `pub mod git_util` (replacing `pub mod git`) and re-exports it as `pub use git_util as git` so `apm_core::git::` paths in external crates continue to resolve without change
- [ ] `gen_hex_id`, `resolve_ticket_branch`, and `branch_name_from_path` are defined in `ticket_fmt.rs` and absent from `git_util.rs`
- [ ] `find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, and `sync_agent_dirs` — plus their private helpers `clean_agent_dirs`, `is_tracked`, and `copy_dir_recursive` — are defined in `worktree.rs` and absent from `git_util.rs`
- [ ] `find_epic_branch`, `find_epic_branches`, `epic_branches`, and `create_epic_branch` are defined in `epic.rs` and absent from `git_util.rs`
- [ ] `merge_into_default` and `pull_default` are defined as `pub fn` in `git_util.rs` and absent from `state.rs`
- [ ] `state.rs` calls `git::merge_into_default` and `git::pull_default` (resolved through the `git_util as git` alias)
- [ ] Every caller of the moved ticket-format functions (`gen_hex_id`, `resolve_ticket_branch`, `branch_name_from_path`) is updated to reference `ticket_fmt::` instead of `git::`
- [ ] Every caller of the moved worktree functions is updated to reference `worktree::` instead of `git::`
- [ ] Every caller of the moved epic functions is updated to reference `epic::` instead of `git::`
- [ ] `cargo build` succeeds with zero errors across `apm-core`, `apm`, and `apm-server`
- [ ] `cargo test` passes (integration suite included)

### Out of scope

- Behaviour changes to any moved function — this is a pure code relocation
- Creating `ticket_fmt.rs` — that is done by ticket 4660b156 (a listed prerequisite)
- Epic business-logic work beyond receiving the four epic functions — handled by ticket eb4789cf
- Further decomposition of `git_util.rs` beyond the functions listed here
- Updating `REFACTOR-CORE.md` or any other documentation
- Changing public API signatures or return types of any moved function

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:21Z | groomed | in_design | philippepascal |