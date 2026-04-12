+++
id = "eb4789cf"
title = "Move epic branch helpers from git.rs into epic.rs"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/eb4789cf-move-epic-branch-helpers-from-git-rs-int"
created_at = "2026-04-12T06:04:33.586819Z"
updated_at = "2026-04-12T06:58:49.401772Z"
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

All changes are in `apm-core/` unless otherwise noted. "Source file" means `git.rs` or `git_util.rs` — use whichever name is present in the branch (b28fe914 renames it).

**1. Cut the four functions from the source file**

From the source file, remove:
- `find_epic_branch` (currently lines 55–71)
- `find_epic_branches` (currently lines 76–102)
- `epic_branches` (currently lines 105–131)
- `create_epic_branch` (currently lines 848–859)

Remove the doc-comment blocks that precede each function.

**2. Paste the four functions into `epic.rs`**

Append them after the existing `create()` function.

`find_epic_branch`, `find_epic_branches`, and `epic_branches` call the private `run()` helper. Because b28fe914 makes `run()` `pub(crate)` in `git_util.rs`, call it as `crate::git_util::run()` — do **not** duplicate the helper in `epic.rs`. Add `use anyhow::Context;` to `epic.rs` if not already present (needed transitively by `run`).

`create_epic_branch` calls `gen_hex_id`, `commit_to_branch`, and `push_branch`. Update each to its fully-qualified crate path:
- `gen_hex_id` → `crate::ticket::gen_hex_id()` (moved to `ticket_fmt.rs` and re-exported through `crate::ticket` by b28fe914)
- `commit_to_branch` → `crate::git_util::commit_to_branch()`
- `push_branch` → `crate::git_util::push_branch()`

**3. Update call sites — 3 files**

`apm/src/cmd/epic.rs` (lines 9, 64, 159, 236):
- `apm_core::git::epic_branches` → `apm_core::epic::epic_branches`
- `apm_core::git::find_epic_branches` → `apm_core::epic::find_epic_branches`

`apm/src/cmd/new.rs` (line 40):
- `git::find_epic_branch` → `epic::find_epic_branch`
- Update the use-import to bring `apm_core::epic` into scope, or switch to the fully-qualified path.

`apm-server/src/main.rs` (lines 162, 288, 311, 341):
- `apm_core::git::find_epic_branch` → `apm_core::epic::find_epic_branch`
- `apm_core::git::epic_branches` → `apm_core::epic::epic_branches`
- `apm_core::git::create_epic_branch` → `apm_core::epic::create_epic_branch`

**4. Verify**

Run `cargo build --workspace` then `cargo test --workspace`. Fix any remaining compilation errors (missed call sites, stale imports).

### Open questions


### Amendment requests

- [x] Remove the duplicated `run()` helper from the Approach. Instead, import `crate::git_util::run()` — ticket b28fe914 will make `run()` `pub(crate)` in `git_util.rs`. Do not duplicate the git invocation wrapper.
- [ ] Fix `gen_hex_id` reference path — the Approach says `create_epic_branch` should call `crate::git::gen_hex_id()`, but by the time this ticket is worked, `gen_hex_id` will have moved to `ticket_fmt.rs` (done by b28fe914). Use `crate::ticket::gen_hex_id()` (via the re-export hub) instead.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:32Z | groomed | in_design | philippepascal |
| 2026-04-12T06:37Z | in_design | specd | claude-0412-0632-93d0 |
| 2026-04-12T06:54Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T06:58Z | ammend | in_design | philippepascal |