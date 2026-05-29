+++
id = "12f2c7fa"
title = "apm refresh-epic: inform by default, add --merge / --pr / --auto modes"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12f2c7fa-apm-refresh-epic-inform-by-default-add-m"
created_at = "2026-05-29T01:17:38.982422Z"
updated_at = "2026-05-29T02:03:56.369400Z"
+++

## Spec

### Problem

The current `apm refresh-epic <id>` always creates a GitHub PR from the default branch into the epic branch. There is no way to check whether main is ahead without making changes, no way to see if a merge would conflict, and no way to perform a local merge without going through GitHub. The quiescence requirement fires even for read-only status checks, which is unnecessarily restrictive.

The command needs explicit mode flags. The default (no flags) should be read-only: report how many commits main is ahead of the epic branch and whether a merge would be clean or conflicted. `--merge` performs a local merge, `--pr` retains the existing GitHub PR behavior, and `--auto` merges locally when clean and falls back to a PR when there are conflicts. The quiescence requirement applies only to the three acting modes (`--merge`, `--pr`, `--auto`), not to the default inform mode. The clean/conflict detection (via `git merge-tree`) is also needed by a separate freshness-surfacing ticket (7a76dd16) and must be extracted into `apm-core` as a reusable helper rather than duplicated.

### Acceptance criteria

- [x] `apm refresh-epic <id>` (no flags) prints the number of commits `main` is ahead of the epic branch and whether a merge would be clean or would conflict; it does not modify any branch, worktree, or PR.
- [x] `apm refresh-epic <id>` (no flags) succeeds regardless of the epic's quiescence state.
- [x] `apm refresh-epic <id>` (no flags) prints "epic branch is up to date with <default_branch>" and exits 0 when `main` has no commits ahead of the epic branch.
- [x] `apm refresh-epic <id> --merge` performs a local merge of `main` into the epic branch; on conflict it aborts the merge and exits with a clear error.
- [x] `apm refresh-epic <id> --merge`, `--pr`, and `--auto` each fail with a clear error when the epic is not quiescent.
- [ ] `apm refresh-epic <id> --pr` opens or updates a PR from `main` into the epic branch (unchanged from current behavior).
- [ ] `apm refresh-epic <id> --auto` merges locally when the merge is clean and falls back to creating or updating a PR when there are conflicts.
- [ ] Passing two or more of `--merge`, `--pr`, `--auto` simultaneously exits with a clear error before doing any git work.
- [ ] A `merge_tree_status` function is exported from `apm-core` and used by `run_refresh_epic`; the logic is not duplicated in the CLI crate.

### Out of scope

- Surfacing freshness data in `apm list`, `apm epic list`, or the web UI (ticket 7a76dd16).
- Auto-detecting staleness at dispatch time or gating ticket dispatch on epic freshness.
- An "accept divergence" mechanism for epics that have commits not yet on main.
- Any changes to `apm epic close`.

### Approach

#### 1. Add `merge_tree_status` to `apm-core/src/epic.rs`

Add a public struct and function:

```rust
pub struct MergeStatus {
    pub ahead: usize,  // commits default_branch has that epic_branch doesn't
    pub clean: bool,   // true iff git merge-tree reports no conflict markers
}

pub fn merge_tree_status(root: &Path, default_branch: &str, epic_branch: &str) -> Result<MergeStatus>
```

Implementation steps:
1. Count ahead commits: run `git log --oneline --no-decorate <epic_branch>..<default_branch>` and count non-empty lines.
2. If `ahead == 0`, return `MergeStatus { ahead: 0, clean: true }` immediately.
3. Compute merge base: run `git merge-base <epic_branch> <default_branch>`.
4. Run `git merge-tree <merge_base> <default_branch> <epic_branch>` (three-argument form; no working-tree changes). Check whether stdout contains `<<<<<<< `. If yes, `clean = false`; otherwise `clean = true`.

Add unit tests in `apm-core/src/epic.rs`:
- Clean merge: commits on `main` that don't overlap with epic — `ahead > 0, clean = true`.
- Conflicting merge: both branches modify the same line — `ahead > 0, clean = false`.
- Up to date: no commits on main ahead — `ahead = 0, clean = true`.

#### 2. Update `RefreshEpic` in `apm/src/main.rs`

Replace the existing single-field variant with:

```rust
RefreshEpic {
    id: String,
    #[arg(long, conflicts_with_all = ["pr", "auto_mode"])]
    merge: bool,
    #[arg(long, conflicts_with_all = ["merge", "auto_mode"])]
    pr: bool,
    #[arg(long = "auto", conflicts_with_all = ["merge", "pr"])]
    auto_mode: bool,
}
```

Clap's `conflicts_with_all` enforces mutual exclusion automatically. Pass all four args through to `run_refresh_epic` in the dispatch arm.

#### 3. Rewrite `run_refresh_epic` in `apm/src/cmd/epic.rs`

New signature: `pub fn run_refresh_epic(root: &Path, id_arg: &str, merge: bool, pr: bool, auto_mode: bool) -> Result<()>`

Logic:
1. Resolve epic branch (unchanged).
2. Derive `epic_id` and load `config` (unchanged).
3. Call `apm_core::epic::merge_tree_status(root, default_branch, &epic_branch)?`.

**Inform mode** (all flags false):
- If `status.ahead == 0`: print "epic branch is up to date with {default_branch}".
- Otherwise: print "{N} commit(s) ahead on {default_branch}; merge would be {clean/conflicted}".
- Return `Ok(())`.

**Acting modes** (any flag set): run quiescence check; bail if not quiescent (unchanged error format).
- If `status.ahead == 0`: print "epic branch is up to date with {default_branch}", return early.

**`--pr` path**: existing behavior — push epic branch, call `gh_pr_create_or_update_between`.

**`--merge` path**: use `apm_core::worktree::find_worktree_for_branch` to find a permanent worktree for the epic branch, falling back to `ensure_worktree` if absent. Call `apm_core::git_util::merge_ref(epic_wt_path, default_branch, &mut messages)`. If the merge returns `None` (conflict path), bail: "merge conflict — resolve manually after checking out {epic_branch}, or use --pr to open a PR instead".

**`--auto` path**: if `status.clean`, run the `--merge` path; otherwise run the `--pr` path.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T01:17Z | — | new | philippepascal |
| 2026-05-29T01:18Z | new | groomed | philippepascal |
| 2026-05-29T01:26Z | groomed | in_design | philippepascal |
| 2026-05-29T01:29Z | in_design | specd | claude |
| 2026-05-29T01:47Z | specd | ready | philippepascal |
| 2026-05-29T02:03Z | ready | in_progress | philippepascal |