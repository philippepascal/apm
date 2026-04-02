+++
id = "645c9f17"
title = "apm clean: add --force flag to skip divergence and merge checks for closed tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "56092"
branch = "ticket/645c9f17-apm-clean-add-force-flag-to-skip-diverge"
created_at = "2026-04-02T05:35:39.235404Z"
updated_at = "2026-04-02T17:09:19.087314Z"
+++

## Spec

### Problem

`apm clean` skips closed tickets in two cases: (1) the local branch tip differs from origin — this happens when `apm state <id> closed` commits to the ticket branch locally but the remote has diverged, or vice versa; (2) the ticket branch was never merged into main — this can happen when a ticket is force-closed without going through the normal implemented → closed path.

Both guards are sensible defaults but become obstacles once a supervisor has verified the tickets are genuinely done and wants to reclaim worktree disk space. There is currently no way to override them short of manually running `git worktree remove --force <path>` and `git branch -D <branch>` for each ticket.

A `--force` flag on `apm clean` should bypass both the divergence check and the merge check for closed tickets, running `git worktree remove --force` and deleting the local branch regardless. It should still only act on tickets in a terminal state — force does not mean "clean everything". 

When using --force, it needs to be in interactive mode, asking the supervisor to approve every `git worktree remove --force`

### Acceptance criteria

- [ ] `apm clean --force` removes the worktree and local branch for a closed ticket whose branch is not merged into main
- [ ] `apm clean --force` removes the worktree and local branch for a closed ticket whose local tip is not an ancestor of the default branch
- [ ] `apm clean --force` removes the worktree and local branch for a closed ticket whose local tip diverges from origin (dirty worktree, tips differ)
- [ ] `apm clean --force` uses `git worktree remove --force` for each worktree removal
- [ ] `apm clean --force` prompts for confirmation before each removal, even when `--yes` is also supplied
- [ ] `apm clean --force` still skips tickets that are not in a terminal state
- [ ] `apm clean --force` still skips tickets with a state mismatch between branch and main
- [ ] `apm clean --force --dry-run` prints what would be removed without modifying anything

### Out of scope

- Bypassing the state-mismatch guard (branch state vs. main state); run `apm sync` to reconcile first
- Bypassing the modified-tracked-files guard; manual cleanup is still required for those
- Bypassing the terminal-state filter; `--force` does not mean "clean all tickets regardless of state"
- Deleting remote branches
- Non-interactive (scriptable) force mode; `--force` always requires a human at the terminal

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:35Z | — | new | apm |
| 2026-04-02T17:00Z | new | groomed | apm |
| 2026-04-02T17:09Z | groomed | in_design | philippepascal |