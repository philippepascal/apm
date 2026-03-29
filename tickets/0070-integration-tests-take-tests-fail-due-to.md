+++
id = 70
title = "Integration tests: take_* tests fail due to stale worktree dirs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0329-1430-main"
branch = "ticket/0070-integration-tests-take-tests-fail-due-to"
created_at = "2026-03-29T23:38:38.394203Z"
updated_at = "2026-03-29T23:38:38.394203Z"
+++

## Spec

### Problem

The take_succeeds_on_ammend_state, take_succeeds_on_blocked_state, and take_appends_handoff_history tests fail when stale worktree directories from previous runs exist at /tmp/.../worktrees/ticket-0001-*. The worktrees dir is relative to the temp git repo dir, and git worktree add fails with 'already exists'. Tests need to clean up or use unique branch names.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:38Z | — | new | claude-0329-1430-main |
