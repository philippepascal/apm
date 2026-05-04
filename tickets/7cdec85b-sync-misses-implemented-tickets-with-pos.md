+++
id = "7cdec85b"
title = "sync misses implemented tickets with post-merge state commits on the ticket branch"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7cdec85b-sync-misses-implemented-tickets-with-pos"
created_at = "2026-05-03T20:15:04.114954Z"
updated_at = "2026-05-04T01:53:58.697984Z"
+++

## Spec

### Problem

sync's merged_into_main() uses 'git branch --merged <default_branch>' to detect merged tickets. This misses any ticket whose branch tip is not an ancestor of main — specifically tickets that had a state-transition commit (e.g. merge_failed → implemented) added to the ticket branch AFTER the implementation content was already merged into main via a manual 'git merge'. The branch tip is ahead of the merge point, so git does not consider it merged, and sync silently skips it.

Observed in ticket 6095305a: content merged at 2442b358 (bdad99da^2), but f88b9ac0 (merge_failed → implemented) was committed to the ticket branch afterward. sync showed 7 of 8 implemented tickets ready to close; 6095305a was invisible.

Two fixes needed:
1. Detection gap: when a ticket branch is not detected as merged by --merged, also check whether the branch tip's *content* (everything up to the merge point bdad99da^2) is present in main, ignoring any trailing state-transition-only commits.
2. User message: when sync sees an implemented ticket it cannot determine was merged, emit a hint such as: 'If this ticket was already merged or you do not want to merge it, close it manually: apm state <id> closed'

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
