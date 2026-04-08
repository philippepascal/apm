+++
id = "645c9f17"
title = "apm clean: add --force flag to skip divergence and merge checks for closed tickets"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "6133"
branch = "ticket/645c9f17-apm-clean-add-force-flag-to-skip-diverge"
created_at = "2026-04-02T05:35:39.235404Z"
updated_at = "2026-04-02T19:07:03.784478Z"
+++

## Spec

### Problem

`apm clean` skips closed tickets in two cases: (1) the local branch tip differs from origin — this happens when `apm state <id> closed` commits to the ticket branch locally but the remote has diverged, or vice versa; (2) the ticket branch was never merged into main — this can happen when a ticket is force-closed without going through the normal implemented → closed path.

Both guards are sensible defaults but become obstacles once a supervisor has verified the tickets are genuinely done and wants to reclaim worktree disk space. There is currently no way to override them short of manually running `git worktree remove --force <path>` and `git branch -D <branch>` for each ticket.

A `--force` flag on `apm clean` should bypass both the divergence check and the merge check for closed tickets, running `git worktree remove --force` and deleting the local branch regardless. It should still only act on tickets in a terminal state — force does not mean "clean everything". 

When using --force, it needs to be in interactive mode, asking the supervisor to approve every `git worktree remove --force`

### Acceptance criteria

- [x] `apm clean --force` removes the worktree and local branch for a closed ticket whose branch is not merged into main
- [x] `apm clean --force` removes the worktree and local branch for a closed ticket whose local tip diverges from origin
- [x] `apm clean --force` uses `git worktree remove --force` for each worktree removal
- [x] `apm clean --force` prompts for confirmation before each removal, even when `--yes` is also supplied
- [x] `apm clean --force` still skips tickets that are not in a terminal state
- [x] `apm clean --force` still skips tickets with a state mismatch between branch and main
- [x] `apm clean --force` still skips tickets with modified tracked files
- [x] `apm clean --force --dry-run` prints what would be removed without modifying anything

### Out of scope

- Bypassing the state-mismatch guard (branch state vs. main state); run `apm sync` to reconcile first
- Bypassing the modified-tracked-files guard; manual cleanup is still required for those
- Bypassing the terminal-state filter; `--force` does not mean "clean all tickets regardless of state"
- Deleting remote branches
- Non-interactive (scriptable) force mode; `--force` always requires a human at the terminal

### Approach

Four files change; changes are additive and do not touch non-force code paths.

**`apm-core/src/git.rs`**
- Change `remove_worktree(root, wt_path)` to `remove_worktree(root, wt_path, force: bool)`; when `force` is true append `"--force"` to the `git worktree remove` args. Update the one existing call site in `clean.rs` to pass `false`.

**`apm-core/src/clean.rs`**
- Change `candidates(root, config)` to `candidates(root, config, force: bool)`.
- When `force=true`, skip the not-merged check and the is-ancestor check.
- When `force=true`, skip the divergence guard.
- When `force=true` and the worktree is dirty (but has no `modified_tracked` files), add the ticket as a normal `CleanCandidate` instead of pushing to `dirty_result`; the force-remove will handle the dirty state. Tickets with `modified_tracked` files still go to `dirty_result` regardless.
- Change `remove(root, candidate)` to `remove(root, candidate, force: bool)`; pass `force` through to `remove_worktree`.

**`apm/src/cmd/clean.rs`**
- Change `run(root, dry_run, yes)` to `run(root, dry_run, yes, force: bool)`.
- Pass `force` to `clean::candidates` and `clean::remove`.
- When `force=true`, always prompt interactively for each `CleanCandidate` regardless of `yes` flag. Use stderr to print a warning line before the prompt (e.g. `"warning: force-removing {branch} — branch may not be merged"`). Accept `y`/`Y` to proceed, anything else skips.
- When `force=true` and not a terminal (no TTY), print an error and return early: `"error: --force requires an interactive terminal"`.
- `--force --dry-run` prints the same "would remove …" lines as without force, no prompts needed.

**`apm/src/main.rs`**
- Add `#[arg(long)] force: bool` to the `Clean` variant (with doc comment: "Bypass merge and divergence checks; always prompts before each removal").
- Update the `long_about` string to document `apm clean --force`.
- Pass `force` to `cmd::clean::run`.

**Tests** (`apm/tests/integration.rs`)
- `clean_force_removes_unmerged_branch` — ticket closed but branch never merged; `--force` removes it after confirming.
- `clean_force_removes_diverged_worktree` — ticket closed, local tip ahead of origin, dirty worktree; `--force` removes it.
- `clean_force_still_skips_non_terminal` — a ticket in `in_progress` state; `--force` does not touch it.
- `clean_force_dry_run_shows_unmerged` — `--force --dry-run` prints "would remove" for an unmerged branch without modifying anything.
- `clean_force_skips_modified_tracked` — ticket closed but worktree has modified tracked files; `--force` does not remove it.

**Order of changes:** git.rs → clean.rs → cmd/clean.rs → main.rs → tests.

### Open questions


### Amendment requests

- [x] Add an explicit Acceptance criterion: "`apm clean --force` still skips tickets with modified tracked files (same behaviour as without `--force`)". This case is mentioned in Out of scope but is not testable from AC alone.
- [x] Merge AC #2 and AC #3 into one — "local tip not an ancestor of the default branch" and "local tip diverges from origin" describe the same guard. Remove the redundant one.
- [x] Remove the specific line number references from the Approach ("lines 127-130", "lines 179-187"). Those will be stale by implementation time. Replace with descriptions of the guard logic (e.g. "the not-merged check", "the is-ancestor check", "the divergence check") so the worker locates them by logic, not line number.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:35Z | — | new | apm |
| 2026-04-02T17:00Z | new | groomed | apm |
| 2026-04-02T17:09Z | groomed | in_design | philippepascal |
| 2026-04-02T17:13Z | in_design | specd | claude-0402-1709-spec1 |
| 2026-04-02T17:27Z | specd | ammend | apm |
| 2026-04-02T17:29Z | ammend | in_design | philippepascal |
| 2026-04-02T17:31Z | in_design | specd | claude-0402-1730-spec2 |
| 2026-04-02T17:39Z | specd | ready | apm |
| 2026-04-02T17:45Z | ready | in_progress | philippepascal |
| 2026-04-02T17:58Z | in_progress | implemented | claude-0402-1800-impl1 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |