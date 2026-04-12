+++
id = "eb4789cf"
title = "Move epic branch helpers from git.rs into epic.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/eb4789cf-move-epic-branch-helpers-from-git-rs-int"
created_at = "2026-04-12T06:04:33.586819Z"
updated_at = "2026-04-12T06:32:39.867887Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
depends_on = ["b28fe914"]
+++

## Spec

### Problem

`epic.rs` currently holds `derive_epic_state()` and `create()`, but the four epic branch discovery and creation functions — `find_epic_branch`, `find_epic_branches`, `epic_branches`, and `create_epic_branch` — live in `git.rs`. These functions are epic-domain operations that happen to call git commands; they are not general-purpose git utilities. Keeping them in `git.rs` obscures where epic logic lives and makes `git.rs` a catch-all rather than a focused module.

The desired state is that all public API touching epic concepts lives in `epic.rs`, while `git.rs` retains only general-purpose git plumbing. Every caller of the four moved functions should be updated to use the `epic::` path. No behaviour changes are permitted.

This ticket covers only the epic-helpers move. The broader `git.rs` reorganisation (rename to `git_util.rs`, extraction of worktree functions, ticket-format helpers, etc.) is handled by sibling tickets in epic 57bce963. Because this ticket depends_on b28fe914, the source file at implementation time may already be named `git_util.rs`; the implementer should use whichever name is present.

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
| 2026-04-12T06:32Z | groomed | in_design | philippepascal |