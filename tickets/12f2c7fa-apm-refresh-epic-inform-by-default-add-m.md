+++
id = "12f2c7fa"
title = "apm refresh-epic: inform by default, add --merge / --pr / --auto modes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12f2c7fa-apm-refresh-epic-inform-by-default-add-m"
created_at = "2026-05-29T01:17:38.982422Z"
updated_at = "2026-05-29T01:26:27.618639Z"
+++

## Spec

### Problem

The current `apm refresh-epic <id>` always creates a GitHub PR from the default branch into the epic branch. There is no way to check whether main is ahead without making changes, no way to see if a merge would conflict, and no way to perform a local merge without going through GitHub. The quiescence requirement fires even for read-only status checks, which is unnecessarily restrictive.

The command needs explicit mode flags. The default (no flags) should be read-only: report how many commits main is ahead of the epic branch and whether a merge would be clean or conflicted. `--merge` performs a local merge, `--pr` retains the existing GitHub PR behavior, and `--auto` merges locally when clean and falls back to a PR when there are conflicts. The quiescence requirement applies only to the three acting modes (`--merge`, `--pr`, `--auto`), not to the default inform mode. The clean/conflict detection (via `git merge-tree`) is also needed by a separate freshness-surfacing ticket (7a76dd16) and must be extracted into `apm-core` as a reusable helper rather than duplicated.

### Acceptance criteria

- [ ] `apm refresh-epic <id>` (no flags) prints the number of commits `main` is ahead of the epic branch and whether a merge would be clean or would conflict; it does not modify any branch, worktree, or PR.
- [ ] `apm refresh-epic <id>` (no flags) succeeds regardless of the epic's quiescence state.
- [ ] `apm refresh-epic <id>` (no flags) prints "epic branch is up to date with <default_branch>" and exits 0 when `main` has no commits ahead of the epic branch.
- [ ] `apm refresh-epic <id> --merge` performs a local merge of `main` into the epic branch; on conflict it aborts the merge and exits with a clear error.
- [ ] `apm refresh-epic <id> --merge`, `--pr`, and `--auto` each fail with a clear error when the epic is not quiescent.
- [ ] `apm refresh-epic <id> --pr` opens or updates a PR from `main` into the epic branch (unchanged from current behavior).
- [ ] `apm refresh-epic <id> --auto` merges locally when the merge is clean and falls back to creating or updating a PR when there are conflicts.
- [ ] Passing two or more of `--merge`, `--pr`, `--auto` simultaneously exits with a clear error before doing any git work.
- [ ] A `merge_tree_status` function is exported from `apm-core` and used by `run_refresh_epic`; the logic is not duplicated in the CLI crate.

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
| 2026-05-29T01:17Z | — | new | philippepascal |
| 2026-05-29T01:18Z | new | groomed | philippepascal |
| 2026-05-29T01:26Z | groomed | in_design | philippepascal |