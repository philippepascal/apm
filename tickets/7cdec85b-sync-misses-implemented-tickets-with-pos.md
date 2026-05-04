+++
id = "7cdec85b"
title = "sync misses implemented tickets with post-merge state commits on the ticket branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7cdec85b-sync-misses-implemented-tickets-with-pos"
created_at = "2026-05-03T20:15:04.114954Z"
updated_at = "2026-05-04T01:55:41.452324Z"
+++

## Spec

### Problem

`sync`'s `merged_into_main()` uses `git branch --merged <default_branch>` (and a squash-merge variant using `git commit-tree` + `git cherry`) to detect ticket branches that have been merged. Both checks compare the **branch tip** against main. Neither handles the case where a state-transition commit was pushed to the ticket branch *after* the implementation content was already merged into main: the tip is no longer an ancestor of main, and the tip tree differs from what was squash-checked, so both detection paths miss the branch silently.

Observed in ticket 6095305a: implementation merged into main at `2442b358` (via merge commit `bdad99da`), then `f88b9ac0` (state: `merge_failed → implemented`) was committed to the ticket branch. `sync` showed 7 of 8 implemented tickets ready to close; 6095305a was invisible because its branch tip is one commit ahead of the merge point.

Two gaps to close:
1. **Detection**: when neither detection path catches a branch, walk back from the tip skipping commits that touch only files under `tickets/`, find the last real-content commit, and re-run the squash check using that commit's tree. If that tree is in main, the branch content was merged.
2. **Fallback message**: when `sync` cannot determine whether an `implemented` ticket was merged (branch exists but no detection path fires), print a hint directing the user to close it manually.

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
| 2026-05-03T20:15Z | — | new | philippepascal |
| 2026-05-04T01:53Z | new | groomed | philippepascal |
| 2026-05-04T01:55Z | groomed | in_design | philippepascal |