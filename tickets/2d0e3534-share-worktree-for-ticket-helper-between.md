+++
id = "2d0e3534"
title = "Share worktree_for_ticket helper between workers.rs and worktrees.rs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2d0e3534-share-worktree-for-ticket-helper-between"
created_at = "2026-04-12T09:02:38.703504Z"
updated_at = "2026-04-12T09:02:38.703504Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

`apm/src/cmd/workers.rs` defines a helper function `worktree_for_ticket()` (lines ~196-213) that resolves a ticket ID to its worktree path. `apm/src/cmd/worktrees.rs` contains similar inline logic (~lines 40-58) for the same purpose but without using the shared helper.

The helper should be moved to a shared location (either `apm/src/util.rs` or as a method on `CmdContext`) so both `workers.rs` and `worktrees.rs` can use it without duplication.

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
| 2026-04-12T09:02Z | — | new | philippepascal |