+++
id = "84e7d15d"
title = "apm sync does not detect squash-merged branches"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "97190"
branch = "ticket/84e7d15d-apm-sync-does-not-detect-squash-merged-b"
created_at = "2026-03-30T20:34:55.205183Z"
updated_at = "2026-03-30T20:35:09.388789Z"
+++

## Spec

### Problem

This was previously ticket #0063 (closed), but the implementation was never merged — the PR was itself squash-merged, which the detection bug caused to be missed, so the fix never landed.

`apm sync` detects merged branches via `git branch --merged`, which only identifies branches whose tip commit is an ancestor of the default branch. Squash merges produce a single new commit in main; the original branch commits are not ancestors, so `merged_into_main()` in `git.rs` misses them.

The existing `squash_merged()` helper in `apm-core/src/git.rs` was added to close this gap, but it uses the wrong algorithm. It runs `git log --cherry-pick --right-only --no-merges origin/main...branch` and treats an empty result as proof that the branch was squash-merged. The `--cherry-pick` flag matches commits by **individual patch-id**. A squash merge creates one combined commit whose patch-id is the aggregate diff of all branch commits — not equal to any individual branch commit's patch-id. So the output is never empty for an actual squash merge, and detection always fails.

Additionally, in the remote path of `merged_into_main()`, candidates are collected only from `origin/ticket/*`. Branches that exist locally but whose remote tracking ref has been deleted (e.g. GitHub auto-deletes the branch after merging) are never checked at all.

Squash-merged tickets are never transitioned to `accepted` and accumulate indefinitely in the branch list, also blocking `apm clean`. GitHub's default merge strategy for most repos is squash merge, making this a near-universal failure mode.

### Acceptance criteria

- [ ] `apm sync` offers to accept a ticket whose branch was squash-merged into the default branch (remote path, remote tracking ref still present)
- [ ] `apm sync` offers to accept a ticket whose branch was squash-merged and the remote tracking ref was subsequently deleted (local branch only remains)
- [ ] `apm sync` does not falsely detect a branch as squash-merged when it has commits not yet present in main
- [ ] `apm sync` continues to detect regular (non-squash) merges as before
- [ ] `cargo test --workspace` passes, including at least one integration test that creates a squash-merge scenario and verifies detection

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T20:34Z | — | new | philippepascal |
| 2026-03-30T20:35Z | new | in_design | philippepascal |