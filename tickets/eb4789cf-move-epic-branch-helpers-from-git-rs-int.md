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

- [ ] `find_epic_branch`, `find_epic_branches`, `epic_branches`, and `create_epic_branch` are defined in `apm-core/src/epic.rs`
- [ ] The four functions are absent from `apm-core/src/git.rs` (or `git_util.rs` if b28fe914 has already landed)
- [ ] `apm/src/cmd/epic.rs` calls `apm_core::epic::epic_branches` instead of `apm_core::git::epic_branches`
- [ ] `apm/src/cmd/epic.rs` calls `apm_core::epic::find_epic_branches` instead of `apm_core::git::find_epic_branches`
- [ ] `apm/src/cmd/new.rs` calls `epic::find_epic_branch` (or equivalent `apm_core::epic::find_epic_branch`) instead of `git::find_epic_branch`
- [ ] `apm-server/src/main.rs` calls `apm_core::epic::find_epic_branch`, `apm_core::epic::epic_branches`, and `apm_core::epic::create_epic_branch` instead of the `apm_core::git::` equivalents
- [ ] `cargo build --workspace` succeeds with zero errors
- [ ] `cargo test --workspace` passes (integration suite included)

### Out of scope

- Renaming `git.rs` to `git_util.rs` — that is done by b28fe914
- Moving any functions other than the four epic-branch helpers out of `git.rs`
- Changing function signatures, return types, or runtime behaviour of the moved functions
- Adding `pub use git_util as git` or any other re-export alias — b28fe914's responsibility
- Extracting worktree or ticket-format helpers from `git.rs` — separate tickets in epic 57bce963
- Updating `REFACTOR-CORE.md` or any other documentation

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