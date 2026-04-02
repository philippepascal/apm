+++
id = "c299b33d"
title = "apm clean: local by default, --remote flag for old branch cleanup"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/c299b33d-apm-clean-local-by-default-remote-flag-f"
created_at = "2026-04-02T20:44:35.825711Z"
updated_at = "2026-04-02T20:50:15.972425Z"
+++

## Spec

### Problem

`apm clean` currently removes both worktrees and local branches, and the two concerns are conflated. In practice, worktree cleanup is a frequent local operation while branch deletion (local and remote) is rarer and more destructive.

Three separate concerns should be cleanly separated:

1. **Worktree removal** (default, local-only): remove the worktree directory for closed/terminal tickets. No branch deletion. Already the safe, frequent operation.

2. **Local branch removal** (`--branches` or folded into default — TBD at spec time): delete local ticket branches for closed tickets. Harmless since the branch content is on the remote.

3. **Remote branch removal** (`--remote`): delete branches from origin that are older than a given threshold. Accepts `--older-than <N>d` (number of days) or `--older-than <date>` (ISO date). Only acts on `ticket/*` branches in terminal states.

4. **Untracked file removal** (`--untracked`): run the equivalent of `git clean -fd` inside each worktree being removed, so worktrees with untracked files (e.g. build artifacts, `.apm-worker.pid`) are not skipped. Without this flag, worktrees with untracked files are left in place with a warning.

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
