+++
id = "6cf21715"
title = "apm verify should detect missing worktree for active-state tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cf21715-apm-verify-should-detect-missing-worktre"
created_at = "2026-04-28T00:50:59.455196Z"
updated_at = "2026-04-28T01:06:11.237378Z"
+++

## Spec

### Problem

`apm verify` currently checks for unknown states, ID/filename mismatches, missing branches on active tickets, merged-but-open branches, and missing spec/history sections. It does not check whether a ticket's worktree directory actually exists on disk.

When a ticket is in `in_design` or `in_progress`, `apm start` has been called and a worktree should be present at `{worktrees_base}/{branch.replace("/", "-")}`. If that directory is deleted (e.g., the repo was re-cloned, the worktrees sibling directory was wiped, or the worktree was force-removed without resetting ticket state), the ticket becomes silently stuck: no agent can work on it and no tooling flags it.

Real incident: ticket ec5e9fe3 was in `in_progress`. `apm worktrees` listed an entry for it at `…/apm--worktrees/ticket-ec5e9fe3-…`. The directory did not exist on disk. `apm verify` ran cleanly and reported no issues.

The fix is to walk every non-terminal ticket whose state is in `{in_design, in_progress}`, compute its expected worktree path, and emit an issue if the directory is absent. `--fix` should not auto-recreate the missing worktree because recreation would silently discard any uncommitted work that may still exist in another clone — a human decision is required (re-provision via `apm start <id>`, or revert state to `ready`).

### Acceptance criteria

- [ ] `apm verify` reports an issue for a ticket in `in_design` state whose branch's expected worktree directory does not exist on disk
- [ ] `apm verify` reports an issue for a ticket in `in_progress` state whose branch's expected worktree directory does not exist on disk
- [ ] The reported issue message for a missing worktree matches the format `#{id} [{state}]: worktree at <path> is missing`
- [ ] `apm verify` does not report a worktree issue for a ticket in `in_design` or `in_progress` state when its worktree directory exists on disk
- [ ] `apm verify` does not report a worktree issue for a ticket in `in_design` or `in_progress` when no `branch` field is set (the existing "state requires branch but none set" issue fires instead)
- [ ] `apm verify` does not report a worktree issue for tickets in states outside `{in_design, in_progress}` (e.g., `specd`, `implemented`, `closed`) even when the computed path is absent
- [ ] `apm verify --fix` does not auto-recreate missing worktrees; the issue is printed and the process exits non-zero, same as without `--fix`

### Out of scope

- Auto-recreating missing worktrees via `--fix`
- Detecting stale git worktree registrations (entries in `git worktree list` that point to deleted directories) — a separate concern
- Worktree checks for `implemented`, `blocked`, or any other state outside `{in_design, in_progress}`
- Recreating or repairing the underlying git metadata for the missing worktree

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:50Z | — | new | philippepascal |
| 2026-04-28T00:51Z | new | groomed | philippepascal |
| 2026-04-28T01:06Z | groomed | in_design | philippepascal |