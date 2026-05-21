+++
id = "7f9c5964"
title = "apm sync should reconcile ticket worktrees, not just bare refs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7f9c5964-apm-sync-should-reconcile-ticket-worktre"
created_at = "2026-05-21T20:51:04.188233Z"
updated_at = "2026-05-21T23:20:59.393499Z"
depends_on = ["9944425e"]
+++

## Spec

### Problem

`apm sync` calls `sync_non_checked_out_refs` (`apm-core/src/git_util.rs`), which deliberately skips every branch that is currently checked out in a worktree. The skip is correct for the single-machine case — updating a ref under an active worktree's HEAD without touching the working tree would leave git's index and HEAD pointing at different commits. But for cross-machine workflows it creates a visible gap.

Concrete scenario: worker on Machine A transitions ticket 996fef40 to `blocked`, commits the state change, and pushes. Machine B runs `apm sync`. The fetch succeeds and `origin/ticket/996fef40-…` advances, but the local worktree at `.apm--worktrees/ticket-996fef40-…` is untouched because that branch is checked out. Machine B's `apm list` already reflects the new state (ticket 9944425e covers the read path), but the working tree inside the worktree still shows the pre-blocked content. Any agent or user who opens the worktree to inspect or continue work sees stale code. The fix is `cd <worktree> && git merge --ff-only origin/<branch>` — a step that is neither obvious nor surfaced anywhere.

The fix is narrowly scoped: when a ticket worktree's branch is strictly behind origin and the working tree is clean (no tracked modifications, staged changes, or non-temp untracked files), `apm sync` should fast-forward it automatically. Worktrees with local work are skipped with a per-worktree warning; worktrees that are ahead or diverged are warned about as today. The safe fast-forward mirrors what `sync_default_branch` already does for the main worktree.

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
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:20Z | groomed | in_design | philippepascal |