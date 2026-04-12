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
| 2026-04-12T06:21Z | groomed | in_design | philippepascal |
