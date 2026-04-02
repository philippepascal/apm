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
| 2026-04-02T20:44Z | — | new | apm |
| 2026-04-02T20:50Z | new | groomed | apm |
| 2026-04-02T20:50Z | groomed | in_design | philippepascal |