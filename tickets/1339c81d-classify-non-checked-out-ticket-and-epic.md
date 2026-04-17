+++
id = "1339c81d"
title = "Classify non-checked-out ticket and epic refs"
state = "in_progress"
priority = 0
effort = 5
risk = 6
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1339c81d-classify-non-checked-out-ticket-and-epic"
created_at = "2026-04-17T18:32:35.787126Z"
updated_at = "2026-04-17T19:13:52.466221Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
depends_on = ["a087593c", "5cf54181"]
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

- [x] `sync_local_ticket_refs` is replaced with `sync_non_checked_out_refs` (or equivalent name) that operates on both `refs/remotes/origin/ticket/*` AND `refs/remotes/origin/epic/*`
- [x] No call path in sync ever rewinds a local ref backward or overwrites a diverged ref — the data-loss bug in the existing unconditional `update-ref` is eliminated
- [x] Branches currently checked out in any worktree are skipped, as today
- [x] For each eligible ref, the five cases are handled exactly: Equal (no-op), Behind (FF via `update-ref`), Ahead (info line only, no push, no ref change), Diverged (warning line, no ref change), RemoteOnly (create local ref at origin SHA)
- [x] Local-only branches (no origin counterpart) are left untouched (no ref change, no push, no warning spam)
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

**1. Reuse the classifier.** Ticket `a087593c` introduces `BranchClass { Equal, Behind, Ahead, Diverged, NoRemote }` and `classify_branch(root, local, remote)` in `apm-core/src/git_util.rs`. This ticket reuses it; add a `RemoteOnly` variant and extend classify to handle the "no local ref" case as well. Depends on A landing first (or coordinate if developed in parallel).

**2. Replace `sync_local_ticket_refs`.** Rewrite as `sync_non_checked_out_refs(root, warnings)` in `apm-core/src/git_util.rs:350`. Change the `for-each-ref` pattern to enumerate both `refs/remotes/origin/ticket/` AND `refs/remotes/origin/epic/`:

```rust
// Two ref namespaces this sync cares about. Both get identical treatment.
const MANAGED_NAMESPACES: &[&str] = &["ticket", "epic"];
```

Iterate each namespace, concatenating results.

**3. Per-ref dispatch.** Skip anything in the checked-out set (logic unchanged from today). For the rest:

```rust
// Classification drives the action. Nothing in this function pushes —
// ahead refs wait for explicit action via apm state transitions.
match classify_branch(root, &format!("refs/heads/{branch}"), &format!("refs/remotes/origin/{branch}")) {
    BranchClass::RemoteOnly => update_ref(...),      // safe: no local commits exist to clobber
    BranchClass::Equal      => {},                   // no-op
    BranchClass::Behind     => update_ref(...),      // safe: local is an ancestor of origin, FF only
    BranchClass::Ahead      => info_line(...),       // critical: do NOT update-ref here — that's the old data-loss bug
    BranchClass::Diverged   => warnings.push(...),   // no ref change, no push
    BranchClass::NoRemote   => {},                   // local-only: leave alone (no auto-push)
}
```

Every non-trivial branch gets a brief comment explaining why; the `Ahead` arm in particular gets a pointed comment referencing the pre-fix bug so future readers don't "simplify" it back into the data-loss shape.

**4. Info and warning messages.** Use the shared guidance module from ticket `5cf54181` for the diverged warning template and the ahead info line. If ticket B lands before C, inline the strings with a `// TODO(5cf54181): move to sync_guidance` marker.

**5. Wiring.** Replace the `git::sync_local_ticket_refs(...)` call in `apm/src/cmd/sync.rs:12` with the new function name. No other call sites.

**6. Tests.** Integration tests in `apm/tests/integration.rs`. Each uses a temp repo with a bare origin; seed different ref states per case:
- `sync_ticket_ref_equal_noop`
- `sync_ticket_ref_behind_ff`
- `sync_ticket_ref_ahead_preserves_local_commits` — seed a local-only commit on a ticket branch, assert it survives sync (regression test for the data-loss bug)
- `sync_ticket_ref_diverged_preserves_local_commits` — same concern, diverged shape
- `sync_ticket_ref_remote_only_creates_local`
- `sync_ticket_ref_local_only_untouched`
- `sync_epic_ref_behind_ff` — covers epic/* namespace extension
- `sync_epic_ref_ahead_preserves_local_commits`
- `sync_checked_out_ticket_skipped` — regression for the existing skip logic

**7. Comments.** Per user feedback, the logic is unintuitive. At minimum: (a) a top-of-function block comment listing the six cases and their actions, (b) an inline comment at the Ahead arm explicitly calling out the pre-fix data-loss bug, (c) direction comments at any `merge-base --is-ancestor` call site.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |
| 2026-04-17T18:42Z | in_design | specd | claude-0417-1645-sync1 |
| 2026-04-17T18:48Z | specd | ready | apm |
| 2026-04-17T19:13Z | ready | in_progress | philippepascal |