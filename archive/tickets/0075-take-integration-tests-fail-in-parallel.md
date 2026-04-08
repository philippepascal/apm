+++
id = 75
title = "take_* integration tests fail in parallel"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "claude-0329-1430-main"
branch = "ticket/0075-take-integration-tests-fail-in-parallel"
created_at = "2026-03-30T00:58:37.658132Z"
updated_at = "2026-03-30T02:41:06.566418Z"
+++

## Spec

### Problem

take_succeeds_on_ammend_state, take_succeeds_on_blocked_state, take_appends_handoff_history fail with 'worktree path already exists' when run concurrently. The tests share a relative worktree path (../worktrees/ticket-...) derived from the tmpdir, which collides when multiple tests share the same /var/folders/.../T/ parent.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T00:58Z | — | new | claude-0329-1430-main |
| 2026-03-30T02:41Z | new | closed | apm |