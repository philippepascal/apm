+++
id = "84e7d15d"
title = "apm sync does not detect squash-merged branches"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
agent = "56990"
branch = "ticket/84e7d15d-apm-sync-does-not-detect-squash-merged-b"
created_at = "2026-03-30T20:34:55.205183Z"
updated_at = "2026-03-31T05:05:14.260003Z"
+++

## Spec

### Problem

This was previously ticket #0063 (closed), but the implementation was never merged — the PR was itself squash-merged, which the detection bug caused to be missed, so the fix never landed.

`apm sync` detects merged branches via `git branch --merged`, which only identifies branches whose tip commit is an ancestor of the default branch. Squash merges produce a single new commit in main; the original branch commits are not ancestors, so `merged_into_main()` in `git.rs` misses them.

The existing `squash_merged()` helper in `apm-core/src/git.rs` was added to close this gap, but it uses the wrong algorithm. It runs `git log --cherry-pick --right-only --no-merges origin/main...branch` and treats an empty result as proof that the branch was squash-merged. The `--cherry-pick` flag matches commits by **individual patch-id**. A squash merge creates one combined commit whose patch-id is the aggregate diff of all branch commits — not equal to any individual branch commit's patch-id. So the output is never empty for an actual squash merge, and detection always fails.

Additionally, in the remote path of `merged_into_main()`, candidates are collected only from `origin/ticket/*`. Branches that exist locally but whose remote tracking ref has been deleted (e.g. GitHub auto-deletes the branch after merging) are never checked at all.

Squash-merged tickets are never transitioned to `accepted` and accumulate indefinitely in the branch list, also blocking `apm clean`. GitHub's default merge strategy for most repos is squash merge, making this a near-universal failure mode.

### Acceptance criteria

- [x] `apm sync` offers to accept a ticket whose branch was squash-merged into the default branch (remote path, remote tracking ref still present)
- [x] `apm sync` offers to accept a ticket whose branch was squash-merged and the remote tracking ref was subsequently deleted (local branch only remains)
- [x] `apm sync` does not falsely detect a branch as squash-merged when it has commits not yet present in main
- [x] `apm sync` continues to detect regular (non-squash) merges as before
- [x] `cargo test --workspace` passes, including at least one integration test that creates a squash-merge scenario and verifies detection

### Out of scope

- Detecting rebase-merged branches (each commit is replayed; `--cherry-pick` already handles this case)
- Changing how accepted or closed tickets are processed after detection
- Handling merge conflicts or partial squash merges
- Performance optimisation of the detection loop

### Approach

**File changed:** `apm-core/src/git.rs`

**Replace the `squash_merged` helper with a correct algorithm.**

The current implementation uses `git log --cherry-pick --right-only --no-merges main...branch`, which matches individual commit patch-ids. A squash merge creates one combined commit whose patch-id is the aggregate diff of all branch commits — not equal to any individual commit patch-id — so the check always fails for real squash merges.

The correct algorithm (used by git-town and similar tools):

1. Find the merge base: `git merge-base <main_ref> <branch>`
2. If the branch tip equals the merge base, skip (already caught by `--merged`).
3. Create a virtual squash commit: `git commit-tree <branch>^{tree} -p <merge_base> -m squash`. This produces a commit object whose patch-id equals the aggregate diff from merge_base to branch tip.
4. Run `git cherry <main_ref> <squash_commit>`. If the output line starts with `-`, main contains a commit with the same aggregate patch-id — the branch was squash-merged.

**Fix ref resolution in the remote path.**

In `merged_into_main()`, the remote path builds candidates from `origin/ticket/*` but strips the `origin/` prefix before passing them to `squash_merged`. When git resolves bare `ticket/foo`, it may not find a local branch and the `merge-base` call fails silently. Pass the full `origin/ticket/foo` ref to `squash_merged`, and strip `origin/` only when adding to the returned list.

**Handle local-only branches in the remote path.**

When GitHub auto-deletes the remote branch after a squash merge, `origin/ticket/foo` disappears but the local `ticket/foo` remains. The remote path currently skips local-only branches entirely. After collecting remote candidates, also collect any local `ticket/*` branches not already in `merged_set` and not already in the remote candidate list. Pass them as bare `ticket/foo` refs (no `origin/` prefix).

**Implementation steps:**

1. Rewrite `squash_merged(root, main_ref, candidates)` with the `commit-tree + cherry` approach. Keep the same function signature; adjust the inner logic only.
2. In `merged_into_main()` remote path:
   a. Pass `origin/ticket/foo` refs (not stripped) to `squash_merged`; strip `origin/` in the returned values before extending `merged`.
   b. After building remote candidates, collect local-only ticket branches and pass them in a second `squash_merged` call with the same `main_ref`.
3. Add integration tests in `apm/tests/integration.rs`:
   - Init a bare "remote" repo and a local clone with the standard `setup()` helper pattern.
   - Create a ticket branch with one commit that puts the ticket in `implemented` state.
   - Squash-merge it into main on the remote side (`git merge --squash` + `git commit`), without merging the original branch commits.
   - Fetch in the local clone (`git fetch`).
   - Call `sync::detect()` and assert the ticket appears in `candidates.accept`.
   - Add a negative test: a ticket branch with commits NOT yet in main should NOT appear in `candidates.accept`.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T20:34Z | — | new | philippepascal |
| 2026-03-30T20:35Z | new | in_design | philippepascal |
| 2026-03-30T20:42Z | in_design | specd | claude-0330-2040-b7f2 |
| 2026-03-30T20:43Z | specd | ready | apm |
| 2026-03-30T20:43Z | ready | in_progress | philippepascal |
| 2026-03-30T20:49Z | in_progress | implemented | claude-0330-2045-x7k2 |
| 2026-03-30T20:51Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |