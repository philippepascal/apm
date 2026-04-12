+++
id = "b28fe914"
title = "Rename git.rs to git_util.rs and extract non-git functions"
state = "ammend"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b28fe914-rename-git-rs-to-git-util-rs-and-extract"
created_at = "2026-04-12T06:04:25.779848Z"
updated_at = "2026-04-12T06:53:56.719184Z"
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

All changes are in `apm-core/` unless otherwise noted. Steps 1–2 update the module registry; steps 3–6 do the moves; step 7 fixes callers; step 8 verifies.

**1. Rename the file**

`git mv apm-core/src/git.rs apm-core/src/git_util.rs`

**2. Update `apm-core/src/lib.rs`**

- Replace `pub mod git;` with `pub mod git_util;`
- Add immediately after: `pub use git_util as git;` — this re-export means all existing `apm_core::git::` paths in `apm`, `apm-server`, and tests continue to resolve for functions that remain in `git_util.rs`. Only callers of *moved* functions need updating.
- Add `pub mod worktree;` (for the new module created in step 3).

**3. Create `apm-core/src/worktree.rs`**

Cut from `git_util.rs` and paste into the new `worktree.rs`:
- Public: `find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, `sync_agent_dirs`
- Private helpers that travel with them: `clean_agent_dirs`, `is_tracked`, `copy_dir_recursive`
- Add necessary imports (mirror what `git.rs` declared for these functions — `std::path::Path`, `crate::config::Config`, etc.).
- `try_worktree_commit` is a private helper used by `commit_to_branch`, which stays in `git_util.rs`; it stays too.

**4. Move ticket-format helpers → `ticket_fmt.rs`**

Cut from `git_util.rs` and paste into the existing `ticket_fmt.rs` (created by 4660b156):
- `gen_hex_id` (the splitmix64-based 8-char hex generator)
- `resolve_ticket_branch`
- `branch_name_from_path`

Add any imports these functions need.

**5. Move epic helpers → `epic.rs`**

Cut from `git_util.rs` and paste into the existing `epic.rs`:
- `find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch`

`create_epic_branch` calls `gen_hex_id` (now in `ticket_fmt`) — update that call to `crate::ticket_fmt::gen_hex_id()`. It also calls `push_branch_tracking` (stays in `git_util`) — keep that call as `crate::git_util::push_branch_tracking(...)` or via the `git` alias.

**6. Move git operations from `state.rs` → `git_util.rs`**

Cut `merge_into_default` and `pull_default` from `state.rs`, change visibility from `fn` to `pub fn`, and append them to `git_util.rs`. Update `state.rs` to call them as `git::merge_into_default(...)` and `git::pull_default(...)`.

**7. Update callers of moved functions**

The `pub use git_util as git` alias handles pure-git callers automatically. Only update callers of the functions that physically moved:

*ticket_fmt functions* (`gen_hex_id`, `resolve_ticket_branch`, `branch_name_from_path`) — change `git::` → `ticket_fmt::`, add `use crate::ticket_fmt;` where absent:
- `apm-core`: `epic.rs:84`, `ticket.rs:455,414,814`, `clean.rs:132`, `start.rs:259,456,499,627,664`, `state.rs:132`
- `apm/src/cmd`: `validate.rs:59`, `assign.rs:78`, `set.rs:33`, `workers.rs:52,208`, `worktrees.rs:48`, `epic.rs:281`, `review.rs:85`, `show.rs:9`, `close.rs:11`, `spec.rs:12`

*worktree functions* — change `git::` → `worktree::`, add `use crate::worktree;` / `use apm_core::worktree;` as appropriate:
- `apm-core/src/state.rs` — `find_worktree_for_branch:304`, `ensure_worktree:246,342`
- `apm-core/src/clean.rs` — `find_worktree_for_branch:151`, `remove_worktree:268`
- `apm-core/src/start.rs` — `find_worktree_for_branch:503,668`
- `apm-core/src/ticket.rs` — `list_ticket_worktrees:809`
- `apm/src/cmd/workers.rs` — `list_ticket_worktrees:24`, `find_worktree_for_branch:210`
- `apm/src/cmd/worktrees.rs` — `find_worktree_for_branch:51`, `remove_worktree:55`
- `apm-server/src/workers.rs` — `list_ticket_worktrees:118`
- `apm/tests/integration.rs` — `find_worktree_for_branch:3171`, `ensure_worktree:3098,3120,3147`

*epic functions* (`find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch`) — change `git::` → `epic::`, add `use apm_core::epic;` where absent:
- `apm/src/cmd/new.rs:40`, `apm/src/cmd/epic.rs:9,64,159,236`
- `apm-server/src/main.rs:161,288,311,341`

**8. Verify**

```
cargo build --workspace
cargo test --workspace
```

Fix any remaining compilation errors (missed call sites, stale imports). The integration test suite is the source of truth.

### Open questions


### Amendment requests

- [ ] Remove worktree extraction from this ticket's scope — creating `worktree.rs` and moving worktree functions (`find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`, `add_worktree`, `remove_worktree`, `sync_agent_dirs`, `clean_agent_dirs`, `is_tracked`, `copy_dir_recursive`) is owned by ticket 4f67992b, which depends on this one. Remove worktree-related acceptance criteria and approach steps.
- [ ] Remove epic helper extraction from this ticket's scope — moving `find_epic_branch`, `find_epic_branches`, `epic_branches`, `create_epic_branch` to `epic.rs` is owned by ticket eb4789cf, which depends on this one. Remove epic-related acceptance criteria and approach steps.
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
