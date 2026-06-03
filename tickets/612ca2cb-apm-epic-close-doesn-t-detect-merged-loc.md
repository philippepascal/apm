+++
id = "612ca2cb"
title = "apm epic close doesn't detect merged (locally) epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/612ca2cb-apm-epic-close-doesn-t-detect-merged-loc"
created_at = "2026-06-03T02:29:52.020160Z"
updated_at = "2026-06-03T06:43:00.715608Z"
+++

## Spec

### Problem

`apm epic close` calls `apm_core::git::is_branch_content_merged` to decide whether the epic branch has been merged into the default branch before deleting it. That function currently prefers `origin/<default>` when the remote ref is present, and only falls back to the local ref when no remote exists.

When a user runs `apm epic submit --merge`, the epic is merged into the **local** `main` but `origin/main` is not updated (the push either hasn't happened or was skipped). At that point `is_branch_content_merged` sees `origin/main` exists, uses it as the reference, and returns `false` — because the epic commits are not yet in `origin/main`. `run_close` then refuses to delete the epic branch with "epic has N commit(s) not yet in main", even though the local merge is complete and the working tree is clean.

The fix is to check local `main` first and treat the branch as merged if its content is present in **either** local `main` or `origin/main`, rather than exclusively preferring the remote.

### Acceptance criteria

- [ ] `apm epic close <id>` succeeds after `apm epic submit --merge` when `origin/main` is behind local `main` (push not yet done).
- [ ] `apm epic close <id>` succeeds when the epic was merged via PR into `origin/main` and local `main` is up to date.
- [ ] `apm epic close <id>` succeeds when the epic was merged via PR and local `main` is behind `origin/main`.
- [ ] `apm epic close <id>` refuses with an "not yet in" error when the epic is present in neither local `main` nor `origin/main`.
- [ ] A unit test in `apm-core/src/git_util.rs` covers: epic merged into local `main`, `origin/main` not updated → `is_branch_content_merged` returns `true`.

### Out of scope

- Changing the commit count in the "not yet in main" error message — the count is cosmetic and not the source of the bug.
- Fixing the `apm epic list` freshness display — the "up to date" label after a local merge is a separate concern.
- Handling the case where `origin/main` is the authoritative source and local `main` should be ignored — that is not a real use case in this workflow.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-03T02:29Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:43Z | groomed | in_design | philippepascal |