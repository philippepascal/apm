+++
id = "bfcaacd0"
title = "e2e tests from 537c2e09 fail on main: setup_merge_dep_repo leaves merge_failed transition on pr_or_epic_merge"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bfcaacd0-e2e-tests-from-537c2e09-fail-on-main-set"
created_at = "2026-08-28T18:18:15.540678Z"
updated_at = "2026-08-28T18:18:45.726142Z"
+++

## Spec

### Problem

cargo test --workspace fails on main: sync_does_not_block_on_closed_branchless_dependency and new_still_fails_when_dependency_does_not_exist_anywhere (apm/tests/e2e.rs ~1338-1450) error with 'inconsistent completion strategies: state.in_progress.transition(implemented) uses merge, state.merge_failed.transition(implemented) uses pr_or_epic_merge'. The helper setup_merge_dep_repo only rewrites the in_progress->implemented completion line (two-space 'completion  = ') and misses merge_failed->implemented ('completion = '). That was valid until c82f853f added Rule 4 (one project-wide completion strategy). 537c2e09 was branched before c82f853f merged, so its tests passed in isolation. Fix: make the helper rewrite both completion lines to merge (or patch the workflow more robustly than string replace).

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
| 2026-08-28T18:18Z | — | new | philippepascal |
| 2026-08-28T18:18Z | new | groomed | philippepascal |
