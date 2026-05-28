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

- [ ] `apm sync` closes an `implemented` ticket whose branch is regular-merged (--no-ff) into the branch named in its `target_branch` field, with close reason `"branch merged into target"`
- [ ] `apm sync` closes an `implemented` ticket whose branch is squash-merged into its `target_branch`
- [ ] `apm sync` does not emit the "close manually" hint for a ticket auto-closed by the new target-branch pass
- [ ] Tickets without a `target_branch` field continue to be detected (or not) exactly as before — no regression in Cases 1, 2, or 3
- [ ] `apm sync` does not error or falsely close a ticket whose `target_branch` value does not exist locally
- [ ] An integration test in `apm/tests/integration.rs` verifies Case 4 for a regular merge into `target_branch`
- [ ] An integration test in `apm/tests/integration.rs` verifies Case 4 for a squash merge into `target_branch`

### Out of scope

- Case 3 analog for `target_branch`: detecting content-merged-but-trailing-state-commits into a non-default target (the `content_merged_into_main` logic extended to arbitrary refs)
- Case 2 analog for `target_branch`: detecting implemented tickets whose branch has been deleted after merging into the target
- Changing how `apm state <id> implemented` routes merges or PR bases
- Remote-only `target_branch` values (refs not fetched locally); those return false and generate no hint

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