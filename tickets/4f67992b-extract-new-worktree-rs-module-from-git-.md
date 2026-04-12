+++
id = "4f67992b"
title = "Extract new worktree.rs module from git.rs, state.rs, and ticket.rs"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4f67992b-extract-new-worktree-rs-module-from-git-"
created_at = "2026-04-12T06:04:31.633559Z"
updated_at = "2026-04-12T06:12:16.212044Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["b28fe914"]
+++

## Spec

### Problem

Worktree lifecycle management is scattered across three modules: `git.rs` (find, list, ensure, add, remove, sync_agent_dirs, copy_dir_recursive), `state.rs` (`provision_worktree`), and `ticket.rs` (`list_worktrees_with_tickets`). There is no single place to understand or modify worktree behavior.

This ticket creates a dedicated `worktree.rs` module that owns the full worktree lifecycle: discovery, creation, provisioning, agent directory syncing, and cleanup.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 3 for the full plan.

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
