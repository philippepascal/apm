+++
id = "7f9c5964"
title = "apm sync should reconcile ticket worktrees, not just bare refs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7f9c5964-apm-sync-should-reconcile-ticket-worktre"
created_at = "2026-05-21T20:51:04.188233Z"
updated_at = "2026-05-21T20:51:08.862158Z"
depends_on = ["9944425e"]
+++

## Spec

### Problem

`apm sync` calls `git_util::sync_non_checked_out_refs` (`apm-core/src/git_util.rs:483`), which explicitly skips any branch currently checked out in a worktree:

```
if checked_out.contains(&branch) {
    continue;
}
```

Comment at line 484-485 says: "These are never touched — they must be managed via the worktree's own git operations." This is deliberate (avoids clobbering uncommitted work) but produces a real cross-machine UX gap.

Concrete reproduction (today, ticket 996fef40):

1. Machine A: worker spawns in `.apm--worktrees/ticket-996fef40-…`, transitions to `blocked`, commits, pushes.
2. Machine B: `apm sync` runs `git fetch` and updates `origin/ticket/996fef40-…`. But the local ticket worktree's HEAD is untouched because the branch is checked out. The worktree's HEAD and working tree still match the pre-`blocked` commit.
3. Machine B's `apm list` shows the new state (ticket 9944425e is about that). But if the user tries to do anything in the worktree (resume work, inspect changes), it sees stale code. Only `cd <worktree> && git pull` reconciles.

Second, more subtle concern: untracked / staged / dirty files in a worktree don't sync at all — git can't push or pull them. A worker that wrote scratch files (`.apm-worker.log`, `pr-body.md`, `ac.txt`, or staged-but-uncommitted code) leaves them stranded on the machine where they were created. Machine B may have a "clean" worktree that's actually missing in-progress work from machine A.

Acceptance:

- `apm sync` (or a new `--worktrees` flag if a behaviour change to default sync is too risky) attempts to fast-forward each ticket worktree whose branch is currently checked out, in cases that are unambiguously safe:
  - Worktree has no uncommitted changes (`git status --porcelain` is empty after filtering known-temp files like `.apm-worker.log`, `.apm-worker.pid`).
  - Branch is `Behind` origin (strict ancestor — same classification used by `sync_non_checked_out_refs`).
  When both hold, run `git -C <worktree> merge --ff-only origin/<branch>` to update HEAD and the working tree.
- When a worktree has local changes (modified tracked files, staged changes, or non-temp untracked files), `apm sync` emits one warning per worktree listing the files and the kinds of divergence found. No automatic action.
- When a worktree is `Ahead` (local commits not pushed) or `Diverged`, same per-worktree warning as today's `sync_non_checked_out_refs` but with the worktree path included so the user knows where to go.
- A summary line at the end of `apm sync` reports counts: N worktrees fast-forwarded, M skipped due to local changes, K diverged.

Out of scope:
- Auto-pushing dirty worktrees or any worktree-modifying operation other than `merge --ff-only`.
- Surfacing worktree state in the UI (could be a follow-up).
- Reconciling untracked files between machines (impossible without rebuilding what tracking-vs-not means).

Related: `9944425e` covers the same UX gap on the read path (`apm list` after `git fetch` doesn't see new origin state because local refs aren't fast-forwarded). This ticket complements it by addressing checked-out branches; together they close the "user ran some sync command, expected to see the truth" surprise.

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
| 2026-05-21T20:51Z | — | new | philippe|philippepascal |