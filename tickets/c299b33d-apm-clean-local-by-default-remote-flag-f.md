+++
id = "c299b33d"
title = "apm clean: local by default, --remote flag for old branch cleanup"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "5780"
branch = "ticket/c299b33d-apm-clean-local-by-default-remote-flag-f"
created_at = "2026-04-02T20:44:35.825711Z"
updated_at = "2026-04-02T20:50:39.619949Z"
+++

## Spec

### Problem

`apm clean` currently removes both worktrees and local branches in a single operation, conflating two concerns with very different frequency and risk profiles. Worktree cleanup is a routine local housekeeping task done regularly after tickets close; branch deletion (local or remote) is rarer and carries more consequence.

The current default is too aggressive: deleting local branches removes the offline reference to merged work and requires a network round-trip to recover. More critically, there is no supported path to delete **remote** branches at all — accumulated `ticket/*` branches on origin grow indefinitely.

The fix is to split `apm clean` into three explicitly opt-in levels:

1. **Worktree removal** (default, no flags): remove the worktree directory under `apm--worktrees/` for each terminal-state ticket. No branch is touched. This is the safe, high-frequency operation.

2. **Local branch removal** (`--branches`): also delete the local `ticket/*` branch. Safe because the content is already on origin, but kept opt-in since losing local refs is annoying when offline.

3. **Remote branch removal** (`--remote --older-than <threshold>`): delete `ticket/*` branches from origin that are in a terminal state and whose last commit predates the given threshold. Requires an explicit age guard to prevent accidental mass deletion.

A fourth flag, `--untracked`, extends worktree removal to cover worktrees that contain untracked non-temp files (build artifacts, etc.) that currently cause a skip-with-warning. Without `--untracked`, only the known-temp files (`.apm-worker.pid`, `.apm-worker.log`, etc.) are auto-removed; all other untracked files block removal with a warning.

### Acceptance criteria

- [ ] **Default behavior (worktrees only):**
- [ ] `apm clean` removes the worktree for each terminal-state ticket that has one
- [ ] `apm clean` does not delete any local branch
- [ ] `apm clean --dry-run` lists worktrees that would be removed and exits without modifying anything
- [ ] `apm clean --dry-run` does not list any local branch for deletion

- [ ] **With `--branches`:**
- [ ] `apm clean --branches` removes worktrees and deletes local `ticket/*` branches for terminal-state tickets
- [ ] `apm clean --branches` prunes the corresponding `origin/<branch>` remote-tracking ref after deleting the local branch (to prevent re-creation on next `apm sync`)
- [ ] `apm clean --branches --dry-run` lists both worktrees and local branches that would be removed

- [ ] **With `--remote --older-than`:**
- [ ] `apm clean --remote --older-than 30d` deletes remote `ticket/*` branches in terminal states whose last commit is older than 30 days
- [ ] `apm clean --remote --older-than 2026-01-01` accepts ISO date (`YYYY-MM-DD`) as the threshold
- [ ] `apm clean --remote` (without `--older-than`) exits with a non-zero status and an error message stating `--older-than` is required
- [ ] `--older-than` without `--remote` exits with a non-zero status and an error message stating it requires `--remote`
- [ ] `apm clean --remote --older-than 30d` only removes branches whose ticket is in a terminal state; non-terminal or non-ticket branches are never touched
- [ ] `apm clean --remote --older-than 30d --yes` skips per-branch confirmation prompts
- [ ] `apm clean --remote --older-than 30d --dry-run` lists remote branches that would be deleted without modifying anything

- [ ] **With `--untracked`:**
- [ ] `apm clean --untracked` removes a worktree that has only untracked non-temp files by deleting those files first, then removing the worktree
- [ ] `apm clean` (without `--untracked`) prints a warning for any worktree with untracked non-temp files and leaves it in place
- [ ] `apm clean --untracked` still skips a worktree that has modified tracked files, printing a warning

- [ ] **Invariants:**
- [ ] Remote branches are never deleted unless `--remote` is explicitly passed
- [ ] Known-temp files (`.apm-worker.pid`, `.apm-worker.log`, `pr-body.md`, `body.md`, `ac.txt`) are auto-removed in all modes without requiring `--untracked`

### Out of scope

- Deleting the ticket file from the `main` branch (that is `apm close`)
- Pruning non-`ticket/*` remote branches
- Any changes to `apm close` behavior
- Configuring default flags via `apm.toml` (e.g. making `--branches` the default per-project)
- Recovering or archiving branches before deletion
- `--remote` without a ticket-state lookup (e.g. deleting any stale remote branch regardless of whether it has a ticket)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:44Z | — | new | apm |
| 2026-04-02T20:50Z | new | groomed | apm |
| 2026-04-02T20:50Z | groomed | in_design | philippepascal |