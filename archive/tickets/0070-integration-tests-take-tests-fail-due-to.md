+++
id = 70
title = "Integration tests: take_* tests fail due to stale worktree dirs"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0070-integration-tests-take-tests-fail-due-to"
created_at = "2026-03-29T23:38:38.394203Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The take_succeeds_on_ammend_state, take_succeeds_on_blocked_state, and take_appends_handoff_history tests fail when stale worktree directories from previous runs exist at /tmp/.../worktrees/ticket-0001-*. The worktrees dir is relative to the temp git repo dir, and git worktree add fails with 'already exists'. Tests need to clean up or use unique branch names.

### Acceptance criteria

- [x] `take_succeeds_on_ammend_state` passes reliably on repeated runs without manual cleanup
- [x] `take_succeeds_on_blocked_state` passes reliably on repeated runs without manual cleanup
- [x] `take_appends_handoff_history` passes reliably on repeated runs without manual cleanup
- [x] All other integration tests continue to pass

### Out of scope

- Changing `setup()` or `setup_with_local_worktrees()` beyond what is needed
- Fixing any other flaky tests

### Approach

`setup_with_local_worktrees()` already exists in `integration.rs` and places the worktrees directory inside the tempdir (so it is cleaned up automatically by `TempDir` drop). The three failing tests currently call `setup()`, which places worktrees at `../worktrees/` relative to the temp repo — outside the tempdir, so they persist across runs and cause `git worktree add` to fail with "already exists" on the second run.

Fix: change the three tests to call `setup_with_local_worktrees()` instead of `setup()`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:38Z | — | new | claude-0329-1430-main |
| 2026-03-30T00:47Z | new | in_design | claude-0329-1430-main |
| 2026-03-30T00:48Z | in_design | specd | claude-0329-1430-main |
| 2026-03-30T00:51Z | specd | ready | apm |
| 2026-03-30T00:55Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:56Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T01:02Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |