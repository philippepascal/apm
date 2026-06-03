+++
id = "612ca2cb"
title = "apm epic close doesn't detect merged (locally) epic"
state = "in_progress"
priority = 0
effort = 1
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/612ca2cb-apm-epic-close-doesn-t-detect-merged-loc"
created_at = "2026-06-03T02:29:52.020160Z"
updated_at = "2026-06-03T20:47:43.796078Z"
+++

## Spec

### Problem

`apm epic close` calls `apm_core::git::is_branch_content_merged` to decide whether the epic branch has been merged into the default branch before deleting it. That function currently prefers `origin/<default>` when the remote ref is present, and only falls back to the local ref when no remote exists.

When a user runs `apm epic submit --merge`, the epic is merged into the **local** `main` but `origin/main` is not updated (the push either hasn't happened or was skipped). At that point `is_branch_content_merged` sees `origin/main` exists, uses it as the reference, and returns `false` — because the epic commits are not yet in `origin/main`. `run_close` then refuses to delete the epic branch with "epic has N commit(s) not yet in main", even though the local merge is complete and the working tree is clean.

The fix is to check local `main` first and treat the branch as merged if its content is present in **either** local `main` or `origin/main`, rather than exclusively preferring the remote.

### Acceptance criteria

- [x] `apm epic close <id>` succeeds after `apm epic submit --merge` when `origin/main` is behind local `main` (push not yet done).
- [x] `apm epic close <id>` succeeds when the epic was merged via PR into `origin/main` and local `main` is up to date.
- [x] `apm epic close <id>` succeeds when the epic was merged via PR and local `main` is behind `origin/main`.
- [x] `apm epic close <id>` refuses with an "not yet in" error when the epic is present in neither local `main` nor `origin/main`.
- [x] A unit test in `apm-core/src/git_util.rs` covers: epic merged into local `main`, `origin/main` not updated → `is_branch_content_merged` returns `true`.

### Out of scope

- Changing the commit count in the "not yet in main" error message — the count is cosmetic and not the source of the bug.
- Fixing the `apm epic list` freshness display — the "up to date" label after a local merge is a separate concern.
- Handling the case where `origin/main` is the authoritative source and local `main` should be ignored — that is not a real use case in this workflow.

### Approach

#### Change `is_branch_content_merged` (apm-core/src/git_util.rs ~line 795)

Replace the current "prefer origin, fall back to local" logic with "check local first, then also check origin":

```rust
pub fn is_branch_content_merged(root: &Path, default_branch: &str, branch: &str) -> Result<bool> {
    // Check local branch first — covers submit --merge before push.
    if is_branch_merged_into(root, branch, default_branch)? {
        return Ok(true);
    }
    // Also check origin/<default_branch> — covers merge-via-PR before local fetch.
    let remote_ref = format!("refs/remotes/origin/{default_branch}");
    if run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
        return is_branch_merged_into(root, branch, &format!("origin/{default_branch}"));
    }
    Ok(false)
}
```

The change is strictly more permissive: returns `true` if the branch content is present in either local `main` or `origin/main`.

#### Add unit test (apm-core/src/git_util.rs, after `is_branch_content_merged_prefers_origin_when_present`)

`is_branch_content_merged_local_merge_origin_behind_returns_true`:
1. `git_init_with_remote()` — local repo with bare remote.
2. Push initial commit to `origin/main`.
3. Create `epic/ff000006-feature`, add a commit, check out `main`.
4. `git merge --no-ff epic/ff000006-feature` into local `main`. **Do not push.**
5. Assert `is_branch_content_merged(p, "main", "epic/ff000006-feature")` returns `true`.

#### Rename existing test

`is_branch_content_merged_prefers_origin_when_present` → `is_branch_content_merged_merged_into_both_returns_true`. The test body is unchanged; only the name is updated to reflect that both refs agree after a push.

#### No changes to apm/src/cmd/epic.rs

`run_close` already calls `is_branch_content_merged` with the right arguments. The bug is entirely inside that function.

#### Existing integration test coverage

`epic_submit_merge_then_close` in `apm/tests/integration.rs` (no-remote repo) already covers the submit-then-close path and will continue to pass. No new integration test is required.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-03T02:29Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:43Z | groomed | in_design | philippepascal |
| 2026-06-03T06:45Z | in_design | specd | claude |
| 2026-06-03T20:47Z | specd | ready | philippepascal |
| 2026-06-03T20:47Z | ready | in_progress | philippepascal |