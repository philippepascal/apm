+++
id = "1469175e"
title = "apm refresh-epic --merge: push after local merge so downstream sees the refresh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1469175e-apm-refresh-epic-merge-push-after-local-"
created_at = "2026-05-31T03:26:11.802159Z"
updated_at = "2026-06-01T02:53:29.892877Z"
+++

## Spec

### Problem

`apm refresh-epic --merge` merges the default branch into the epic worktree locally but does not push to origin. The dispatch path in `apm start` calls `remote_branch_tip`, which prefers `origin/<epic-branch>` when that ref exists. Any ticket dispatched after a local-only merge therefore receives the pre-merge epic content. The refresh is silently ineffective for all downstream workers until the supervisor pushes manually.

This asymmetry was confirmed in practice on the syn project: `apm refresh-epic <id> --merge` completed successfully, but a subsequent `apm start` on a ticket in that epic dispatched from the stale `origin/<epic-branch>` tip. The `--pr` path (lines 203–225 of `apm/src/cmd/epic.rs`) already calls `push_branch_tracking` before opening the PR; the `--merge` path has no equivalent step.

### Acceptance criteria

- [ ] `apm refresh-epic <id> --merge --push` pushes the epic branch to origin after a successful local merge; `git rev-parse origin/<epic-branch>` equals the post-merge local tip.
- [ ] `apm refresh-epic <id> --merge --no-push` completes the local merge without pushing; `origin/<epic-branch>` is unchanged; a warning is printed to stderr stating that downstream `apm start` will read stale content until the branch is pushed manually.
- [ ] `apm refresh-epic <id> --merge` with stdout connected to a terminal prompts `Push refreshed epic to origin? [Y/n]`; pressing Enter or typing `y`/`Y` pushes; typing `n`/`N` skips with the stale-origin warning.
- [ ] `apm refresh-epic <id> --merge` with stdout not connected to a terminal skips the push without prompting and prints the stale-origin warning to stderr.
- [ ] When the local merge fails with a conflict, no push is attempted regardless of the `--push`/`--no-push` flags.
- [ ] Passing both `--push` and `--no-push` together is rejected as a CLI error.
- [ ] The `--pr` path behaviour is unchanged: `push_branch_tracking` still runs before PR creation.
- [ ] The default path (no `--merge`, `--pr`, or `--auto` flag) behaviour is unchanged.

### Out of scope

- Cascading default-branch updates into individual in-flight ticket worktrees after an epic refresh — separate, larger concern; file as its own ticket
- Changes to the dispatch-time merge logic in `apm-core/src/start.rs` (`remote_branch_tip`'s origin-preference) — separate design decision
- `--push`/`--no-push` flags on the `--pr` path (`--pr` always pushes before creating a PR; no change needed)
- Explicit push-flag support for the `--auto` path — the default prompt/warn logic applies when `--auto` resolves to a local merge

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:53Z | groomed | in_design | philippepascal |