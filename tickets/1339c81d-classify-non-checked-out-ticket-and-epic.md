+++
id = "1339c81d"
title = "Classify non-checked-out ticket and epic refs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1339c81d-classify-non-checked-out-ticket-and-epic"
created_at = "2026-04-17T18:32:35.787126Z"
updated_at = "2026-04-17T18:34:19.027296Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
+++

## Spec

### Problem

`sync_local_ticket_refs` in `apm-core/src/git_util.rs:350` unconditionally `update-ref`s every non-checked-out `ticket/*` ref to its origin SHA. This is a latent data-loss bug: if a local ticket branch has commits that aren't on origin (e.g. committed but never pushed), sync silently rewinds the local ref to the origin SHA, orphaning those commits.

It also ignores `epic/*` branches entirely — they are never fetched-forward, never warned about, and drift stale relative to origin.

Per the review decision captured in the design doc, **no automatic pushes**: ahead branches get an info line only, not a push. Divergence is reported, not clobbered. Local-only branches (no remote counterpart) are left alone.

Sync's job for non-checked-out `ticket/*` and `epic/*` refs is:
- Equal → no-op
- Behind (FF possible) → fast-forward via `update-ref`
- Ahead → info line only, no push, no clobber
- Diverged → warn, skip, no clobber
- Remote-only → create local ref at origin SHA
- Local-only → leave alone

See `/Users/philippepascal/Documents/apm/apm-sync-scenarios.md` for the full scenario matrix and algorithm. Implementers must add comments explaining the classification states and why each maps to its action — the logic is not intuitive at a glance, especially around ancestry-check direction and the data-loss fix.

### Acceptance criteria

- [ ] `sync_local_ticket_refs` is replaced with `sync_non_checked_out_refs` (or equivalent name) that operates on both `refs/remotes/origin/ticket/*` AND `refs/remotes/origin/epic/*`
- [ ] No call path in sync ever rewinds a local ref backward or overwrites a diverged ref — the data-loss bug in the existing unconditional `update-ref` is eliminated
- [ ] Branches currently checked out in any worktree are skipped, as today
- [ ] For each eligible ref, the five cases are handled exactly: Equal (no-op), Behind (FF via `update-ref`), Ahead (info line only, no push, no ref change), Diverged (warning line, no ref change), RemoteOnly (create local ref at origin SHA)
- [ ] Local-only branches (no origin counterpart) are left untouched (no ref change, no push, no warning spam)
- [ ] `epic/*` refs receive identical treatment to `ticket/*` refs; integration tests cover at least one `epic/*` scenario in each non-trivial case
- [ ] The module carries block comments documenting the classification states and explicit direction of ancestry checks
- [ ] Integration tests in `apm/tests/integration.rs` cover: equal, behind-FF, ahead-no-clobber, diverged-no-clobber, remote-only-create, local-only-untouched — for both `ticket/*` and at least one representative `epic/*` case
- [ ] `cargo test --workspace` passes

### Out of scope

- Main branch handling (ticket `a087593c`)
- Mid-merge / mid-rebase detection (ticket `5cf54181`)
- Any form of automatic `git push` from `apm sync`
- Publishing local-only branches that have no origin counterpart — those require an explicit user action (or a future opt-in flag)
- Touching branches currently checked out in any worktree (still skipped)
- Rewriting history on diverged refs or offering auto-rebase
- Changes to `apm state <id> implemented` push behavior

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |