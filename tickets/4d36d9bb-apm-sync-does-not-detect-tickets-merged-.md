+++
id = "4d36d9bb"
title = "apm sync does not detect tickets merged into their target branch"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d36d9bb-apm-sync-does-not-detect-tickets-merged-"
created_at = "2026-05-28T20:46:27.893432Z"
updated_at = "2026-05-28T20:46:44.798330Z"
+++

## Spec

### Problem

Since b0ea6a04 (April 3), the Merge completion strategy routes tickets with target_branch set into that branch (e.g. an epic branch) rather than always into main. sync::detect only checks merges into the default branch (Cases 1, 2, 3 all use merged_into_main / content_merged_into_main). Tickets merged into an epic branch stay in implemented state forever and now emit a spurious hint asking the supervisor to close them manually. The fix: in sync::detect, after the three existing passes, add a pass that reads each remaining implemented ticket's target_branch field; if set, checks whether the ticket branch is merged into that target (using git::is_ancestor or merged_into equivalent); and if so, adds it to merged_set (suppressing the hint) and to close candidates (auto-closing it the same way Case 1 does). This mirrors exactly what sync already does for main-merged tickets. Root cause traced to b0ea6a04 changing Merge strategy to honor target_branch without a corresponding sync update. The hint generation added in 14338748 (May 3) made the gap visible.

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
