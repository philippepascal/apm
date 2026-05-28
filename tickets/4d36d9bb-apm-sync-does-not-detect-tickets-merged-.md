+++
id = "4d36d9bb"
title = "apm sync does not detect tickets merged into their target branch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d36d9bb-apm-sync-does-not-detect-tickets-merged-"
created_at = "2026-05-28T20:46:27.893432Z"
updated_at = "2026-05-28T20:46:53.536986Z"
+++

## Spec

### Problem

Since b0ea6a04 (April 3), the Merge completion strategy routes tickets with `target_branch` set into that branch (e.g. an epic branch) rather than always into main. `sync::detect` was not updated alongside that change: all three existing passes (Cases 1, 2, 3) check merges only against the project's default branch. Tickets merged into an epic branch therefore stay in `implemented` state permanently and, since 14338748 (May 3), emit a spurious hint asking the supervisor to close them manually.

The fix adds a Case 4 after Case 3 in `sync::detect`. It iterates every `implemented` ticket branch that the earlier passes did not recognise; for any that carry a non-empty `target_branch` field, it checks whether that branch has been merged — by regular merge or squash — into the named target. Matches are added to `merged_set` (suppressing the hint) and to the close-candidates list, mirroring exactly what Case 1 already does for main-merged tickets.

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
| 2026-05-28T20:46Z | — | new | philippepascal |
| 2026-05-28T20:46Z | new | groomed | philippepascal |
| 2026-05-28T20:46Z | groomed | in_design | philippepascal |