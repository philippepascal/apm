+++
id = "4f67992b"
title = "Extract new worktree.rs module from git.rs, state.rs, and ticket.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4f67992b-extract-new-worktree-rs-module-from-git-"
created_at = "2026-04-12T06:04:31.633559Z"
updated_at = "2026-04-12T06:27:43.623452Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:27Z | groomed | in_design | philippepascal |