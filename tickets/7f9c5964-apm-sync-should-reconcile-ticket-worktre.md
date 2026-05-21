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

- [ ] When a ticket worktree's branch is `Behind` origin and the worktree has no uncommitted changes (excluding `.apm-worker.log` and `.apm-worker.pid`), `apm sync` runs `git merge --ff-only origin/<branch>` in that worktree and prints one confirmation line per fast-forwarded worktree.
- [ ] When a ticket worktree has uncommitted changes (tracked modifications, staged changes, or non-temp untracked files), `apm sync` emits one warning per worktree that names the worktree path and lists the dirty files, and skips the fast-forward.
- [ ] When a ticket worktree's branch is `Ahead` of origin, `apm sync` emits a per-worktree info line that includes the worktree path and takes no other action.
- [ ] When a ticket worktree's branch has `Diverged` from origin, `apm sync` emits a per-worktree warning that includes the worktree path and takes no other action.
- [ ] After processing all worktrees, `apm sync` prints a summary: `N worktree(s) fast-forwarded, M skipped (local changes), K skipped (ahead/diverged)` — omitting zero-count terms.
- [ ] When no ticket worktrees exist, no worktree-related lines appear in `apm sync` output.
- [ ] All per-worktree lines and the summary line are suppressed when `--quiet` is passed.
- [ ] Worktree reconciliation runs in the same `!offline` block as `sync_non_checked_out_refs`; passing `--offline` skips it entirely.

### Out of scope

- Auto-pushing dirty or ahead worktrees, or any worktree modification other than `git merge --ff-only`.
- `epic/*` worktrees (rare in practice; can be added in a follow-up if needed; `list_ticket_worktrees` only returns `ticket/*` branches).
- Adding a `--worktrees` / `--no-worktrees` CLI flag (default behavior change is safe; a flag can be added as a follow-up if operators want opt-out).
- Surfacing per-worktree sync state in the web UI (follow-up).
- Reconciling untracked non-temp files between machines (impossible without a file-tracking layer outside git).
- Worktrees in `Equal`, `NoRemote`, or `RemoteOnly` states (these are silent no-ops today and remain so).

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