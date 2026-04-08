+++
id = "d4b14b15"
title = "apm sync: update local ticket branch refs after remote operations"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "44329"
branch = "ticket/d4b14b15-apm-sync-update-local-ticket-branch-refs"
created_at = "2026-03-31T05:10:30.606044Z"
updated_at = "2026-03-31T05:42:51.261916Z"
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

- [x] After `apm sync` fetches new commits on a ticket branch pushed by another agent, `apm clean` no longer emits a "local tip differs from origin" warning for that branch
- [x] After `apm sync`, a ticket branch that exists only on origin (no local ref yet) gains a local ref pointing to the origin tip
- [x] After `apm sync`, a ticket branch whose local ref was already equal to origin is left unchanged
- [x] `apm sync` does not update the local ref for a branch that is currently checked out in a permanent worktree
- [x] `apm sync` does not update the local ref for the branch currently checked out in the main worktree
- [x] When `git update-ref` fails for a single branch, `apm sync` emits a warning to stderr and continues without aborting the overall sync
- [x] `apm sync --offline` (no fetch) does not call the local-ref update logic

### Out of scope

- Updating local refs after operations other than `apm sync` (individual `apm state`, `apm spec`, `apm close` commands do not need this; the divergence is resolved on the next `apm sync`)
- Handling non-ticket branches (only `ticket/*` namespace)
- Force-updating local refs when they are ahead of origin (this ticket only handles origin-ahead or origin-equal cases that are visible after fetch)
- Any changes to `apm clean`'s comparison logic — the fix is upstream in sync, not in clean

### Approach

**New function in `apm-core/src/git.rs`:** `pub fn sync_local_ticket_refs(root: &Path)`

1. Get all branches currently checked out across all worktrees:
   - Run `git worktree list --porcelain` (reuse the same porcelain parsing already used by `find_worktree_for_branch` and `list_ticket_worktrees`)
   - Collect every `branch refs/heads/<name>` line into a `HashSet<String>` of branch names
   - This covers the main worktree, permanent ticket worktrees, and any temporary worktrees

2. Enumerate all origin ticket branches:
   - Run `git for-each-ref --format=%(refname:short) refs/remotes/origin/ticket/`
   - Each result is of the form `origin/ticket/<slug>`, strip the `origin/` prefix to get the local branch name `ticket/<slug>`

3. For each origin ticket branch:
   - If `ticket/<slug>` is in the checked-out set: skip silently
   - Resolve the origin SHA: `git rev-parse refs/remotes/origin/ticket/<slug>` — if this fails (ref vanished between enumeration and resolve): skip silently
   - Call `git update-ref refs/heads/ticket/<slug> <sha>` — this creates the local ref if absent or advances/sets it if present; it is intentionally unconditional for non-checked-out branches
   - If `update-ref` fails: `eprintln!("warning: could not update local ref {branch}: {e:#}")` and continue; do not propagate the error

4. Function signature returns `()` (not `Result`) — all failures are handled internally as warnings

**Call site in `apm/src/cmd/sync.rs`:**

In the `Ok(_)` arm of `git::fetch_all` (currently lines 11–17), add a call to `git::sync_local_ticket_refs(root)` immediately after the fetch succeeds. Do not call it in the offline path.

```rust
if !offline {
    match git::fetch_all(root) {
        Ok(_) => {
            git::sync_local_ticket_refs(root);  // new
        }
        Err(e) => {
            eprintln!("warning: fetch failed (no remote configured?): {e:#}");
        }
    }
}
```

**Tests to add in `apm-core/tests/` or inline in `git.rs`:**

- Integration test: create a bare origin, two clones; clone A pushes a new commit to `ticket/xxx`; clone B fetches (via `fetch_all`) then `sync_local_ticket_refs`; assert clone B's local `refs/heads/ticket/xxx` equals origin's tip
- Test: branch checked out in a worktree is skipped (local ref unchanged after sync_local_ticket_refs)
- Test: branch not present locally before sync gains a local ref

No new files needed — the function lives alongside `push_ticket_branches` and related helpers in `git.rs`.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T05:10Z | — | new | apm |
| 2026-03-31T05:10Z | new | in_design | apm |
| 2026-03-31T05:25Z | in_design | new | apm |
| 2026-03-31T05:25Z | new | in_design | philippepascal |
| 2026-03-31T05:32Z | in_design | specd | claude-0330-spec-d4b1 |
| 2026-03-31T05:35Z | specd | ready | apm |
| 2026-03-31T05:35Z | ready | in_progress | philippepascal |
| 2026-03-31T05:39Z | in_progress | implemented | claude-0330-1430-d4b1 |
| 2026-03-31T05:41Z | implemented | accepted | apm-sync |
| 2026-03-31T05:42Z | accepted | closed | apm-sync |