+++
id = "d4b14b15"
title = "apm sync: update local ticket branch refs after remote operations"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/d4b14b15-apm-sync-update-local-ticket-branch-refs"
created_at = "2026-03-31T05:10:30.606044Z"
updated_at = "2026-03-31T05:25:46.637659Z"
+++

## Spec

### Problem

After any operation that commits to a ticket branch and pushes to origin (state transitions, close, accept, spec writes via apm spec), the local branch ref is not updated — only origin/ticket/... advances. This causes apm clean to emit "local tip differs from origin" warnings and skip those branches, making clean effectively a no-op until the user manually fetches.

The fix is to update local branch refs to match origin after each push. This must handle several scenarios correctly:

Happy paths:
- Branch exists locally and is not checked out anywhere: update-ref to match origin
- Branch exists locally and is checked out in a worktree: skip (git refuses to update checked-out refs; the worktree HEAD stays authoritative)
- Branch does not exist locally yet: create local ref pointing to origin tip
- apm sync running a full refresh: update all local refs in one pass, skipping checked-out ones

Sad paths:
- Origin push failed (network error, rejected): do not update local ref — local and origin are already consistent at the pre-push tip
- origin/ticket/... does not exist after push (very unlikely, but guard it): skip silently
- Branch is checked out in a worktree AND has uncommitted changes: skip — no local ref update should be attempted, worktree state is preserved
- Branch is checked out in the main worktree (rare but possible if user ran git checkout manually): skip
- Multiple worktrees have the same branch checked out (should not happen, but guard it): skip
- update-ref fails for any unexpected reason: warn and continue, do not abort the parent operation

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T05:10Z | — | new | apm |
| 2026-03-31T05:10Z | new | in_design | apm |
| 2026-03-31T05:25Z | in_design | new | apm |
| 2026-03-31T05:25Z | new | in_design | philippepascal |
