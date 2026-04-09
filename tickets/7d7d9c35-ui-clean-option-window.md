+++
id = "7d7d9c35"
title = "UI: clean option window"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7d7d9c35-ui-clean-option-window"
created_at = "2026-04-09T05:15:30.189617Z"
updated_at = "2026-04-09T05:23:11.521685Z"
+++

## Spec

### Problem

The clean command exposes seven flags (--dry-run, --force, --branches, --remote, --older-than, --untracked, --yes) and produces detailed log output (per-worktree, per-branch, per-remote-branch messages plus warnings). The existing Clean button in the SupervisorView header fires a hard-coded POST /api/clean with no options and discards all log output — the server handler ignores every flag and returns only a removed count.

Users who need to preview what clean will do (dry-run), delete local branches alongside worktrees (--branches), clean remote branches (--remote --older-than), force-remove unmerged worktrees (--force), or remove untracked files first (--untracked) must fall back to the CLI. Log output produced during dry-run is completely invisible in the UI.

The fix is to replace the one-click Clean button with a modal window that exposes all options and displays the command log output, making dry-run a first-class UI workflow.

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
| 2026-04-09T05:15Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:23Z | groomed | in_design | philippepascal |