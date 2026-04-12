+++
id = "b28fe914"
title = "Rename git.rs to git_util.rs and extract non-git functions"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b28fe914-rename-git-rs-to-git-util-rs-and-extract"
created_at = "2026-04-12T06:04:25.779848Z"
updated_at = "2026-04-12T07:06:34.929268Z"
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
- [ ] `merge_into_default` and `pull_default` are defined as `pub fn` in `git_util.rs` and absent from `state.rs`
- [ ] The private `run()` helper in `git_util.rs` is declared `pub(crate)` so downstream modules (`worktree.rs`, `epic.rs`) can call it without duplication
- [ ] `state.rs` calls `git::merge_into_default` and `git::pull_default` (resolved through the `git_util as git` alias)
- [ ] Every caller of the moved ticket-format functions (`gen_hex_id`, `resolve_ticket_branch`, `branch_name_from_path`) is updated to reference `ticket_fmt::` instead of `git::`
- [ ] `cargo build` succeeds with zero errors across `apm-core`, `apm`, and `apm-server`
- [ ] `cargo test` passes (integration suite included)

### Out of scope

- Behaviour changes to any moved function — this is a pure code relocation
- Creating `ticket_fmt.rs` — that is done by ticket 4660b156 (a listed prerequisite)
- Creating `worktree.rs` and moving worktree functions (`find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, `sync_agent_dirs`, and their private helpers `clean_agent_dirs`, `is_tracked`, `copy_dir_recursive`) — owned by ticket 4f67992b, which depends on this one
- Moving epic helpers (`find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch`) to `epic.rs` — owned by ticket eb4789cf, which depends on this one
- Further decomposition of `git_util.rs` beyond the functions listed here
- Updating `REFACTOR-CORE.md` or any other documentation
- Changing public API signatures or return types of any moved function

### Approach

All changes are in `apm-core/` unless otherwise noted.

**1. Rename the file**

Run `git mv apm-core/src/git.rs apm-core/src/git_util.rs`.

**2. Update `apm-core/src/lib.rs`**

- Replace `pub mod git;` with `pub mod git_util;`.
- Add immediately after: `pub use git_util as git;` — this re-export means all existing `apm_core::git::` paths in `apm`, `apm-server`, and tests continue to resolve for functions that remain in `git_util.rs`. Only callers of *moved* functions need updating.

**3. Move ticket-format helpers → `ticket_fmt.rs`**

Cut from `git_util.rs` and paste into the existing `ticket_fmt.rs` (created by 4660b156):
- `gen_hex_id` (the splitmix64-based 8-char hex generator)
- `resolve_ticket_branch`
- `branch_name_from_path`

Add any imports these functions need (mirror what they used in `git_util.rs`).

**4. Absorb git operations from `state.rs` into `git_util.rs`**

Cut `merge_into_default` and `pull_default` from `state.rs`, change their visibility from `fn` to `pub fn`, and append them to `git_util.rs`. Update the call sites inside `state.rs` to use `git::merge_into_default(...)` and `git::pull_default(...)` (resolved via the alias from step 2).

**5. Make `run()` pub(crate) in `git_util.rs`**

Locate the private `run()` helper in `git_util.rs` and change its declaration from `fn run(` to `pub(crate) fn run(`. This allows downstream modules (`worktree.rs`, `epic.rs`) created by dependent tickets to reuse it without duplication.

**6. Update callers of moved ticket-format functions**

The `pub use git_util as git` alias handles all callers of functions that *stayed* in `git_util.rs` automatically. Only callers of `gen_hex_id`, `resolve_ticket_branch`, and `branch_name_from_path` need updating: change `git::` to `ticket_fmt::` and add `use crate::ticket_fmt;` (or `use apm_core::ticket_fmt;` for crates outside `apm-core`) where absent.

Files in `apm-core/src/` that reference these three functions: `epic.rs`, `ticket.rs`, `clean.rs`, `start.rs`, `state.rs`.

Files in `apm/src/cmd/` that reference these functions: `validate.rs`, `assign.rs`, `set.rs`, `workers.rs`, `worktrees.rs`, `epic.rs`, `review.rs`, `show.rs`, `close.rs`, `spec.rs`.

**7. Verify**

Run `cargo build --workspace` then `cargo test --workspace`. Fix any remaining compilation errors (missed call sites, stale imports). The integration test suite is the source of truth.

### Open questions


### Amendment requests

- [x] Remove worktree extraction from this ticket's scope — creating `worktree.rs` and moving worktree functions (`find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, `sync_agent_dirs`, `clean_agent_dirs`, `is_tracked`, `copy_dir_recursive`) is owned by ticket 4f67992b, which depends on this one. Remove worktree-related acceptance criteria and approach steps.
- [x] Remove epic helper extraction from this ticket's scope — moving `find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch` to `epic.rs` is owned by ticket eb4789cf, which depends on this one. Remove epic-related acceptance criteria and approach steps.
- [ ] This ticket's scope should be: (1) rename git.rs → git_util.rs, (2) add `pub use git_util as git` alias in lib.rs, (3) move `gen_hex_id`, `resolve_ticket_branch`, `branch_name_from_path` to `ticket_fmt.rs`, (4) absorb `merge_into_default` and `pull_default` from `state.rs`. Nothing else.
- [ ] Remove specific line number citations from the Approach — use function names as anchors instead, since line numbers drift as other tickets land.
- [ ] Make the private `run()` helper `pub(crate)` in `git_util.rs` so that downstream modules (`worktree.rs`, `epic.rs`) can import it rather than duplicating it.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:21Z | groomed | in_design | philippepascal |
| 2026-04-12T06:27Z | in_design | specd | claude-0412-0621-6f10 |
| 2026-04-12T06:53Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T07:06Z | ammend | in_design | philippepascal |