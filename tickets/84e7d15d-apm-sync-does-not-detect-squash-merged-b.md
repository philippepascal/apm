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

`apm sync` detects merged branches via `git branch --merged`, which only identifies branches whose tip commit is an ancestor of the default branch. Squash merges produce a single new commit in main; the original branch commits are not ancestors, so `merged_into_main()` in `git.rs` misses them. Squash-merged tickets are never transitioned to `accepted` and accumulate indefinitely in the branch list, also blocking `apm clean`.

GitHub's default merge strategy for most repos is squash merge, making this a common failure mode.

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
| 2026-03-30T20:34Z | — | new | philippepascal |
| 2026-03-30T20:35Z | new | in_design | philippepascal |