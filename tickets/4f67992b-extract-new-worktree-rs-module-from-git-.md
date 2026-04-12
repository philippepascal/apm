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

Checkboxes; each one independently testable.

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
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:27Z | groomed | in_design | philippepascal |