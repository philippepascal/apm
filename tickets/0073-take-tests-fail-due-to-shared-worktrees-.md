+++
id = 73
title = "take tests fail due to shared worktrees dir collision"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0329-1430-main"
branch = "ticket/0073-take-tests-fail-due-to-shared-worktrees-"
created_at = "2026-03-29T23:59:00.212794Z"
updated_at = "2026-03-29T23:59:00.212794Z"
+++

## Spec

### Problem

take_succeeds_on_ammend_state, take_succeeds_on_blocked_state, take_appends_handoff_history fail because ../worktrees/ticket-0001-* paths resolve outside the tempdir and persist across runs. Leftover dirs from prior runs cause git worktree add to fail with 'already exists'. Test setup should place the worktrees dir inside the temp dir.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:59Z | — | new | claude-0329-1430-main |
