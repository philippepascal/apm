+++
id = "992d816e"
title = "apm sync hint wrong epic to close"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/992d816e-apm-sync-hint-wrong-epic-to-close"
created_at = "2026-06-03T02:27:42.503993Z"
updated_at = "2026-06-03T22:22:00.760854Z"
+++

## Spec

### Problem

When `apm sync` computes which epics to list as "ready to close", it calls `is_branch_content_merged(root, default_branch, epic_branch)` for each epic branch. That function checks first whether `epic_branch` is a git ancestor of main (`git merge-base --is-ancestor epic main`). For any epic branch that was created from an old commit of main but never had development committed to it, the branch tip IS a literal ancestor of main — so the function returns `true` and the epic is added to `epic_close_hints`.

The result is that every stale, undeveloped epic branch (visible in `apm epic list` as `↓N clean`) is incorrectly listed as "Epics ready to close (apm epic close <id>)", while epics that have actual unmerged work (like the `done` epic whose branch is ahead of main) are correctly omitted. The user is prompted to run `apm epic close` on in-progress epics that have open tickets and no merged content.

### Acceptance criteria

- [x] An epic branch that has no commits beyond its merge-base with main is NOT listed in "Epics ready to close" by `apm sync`
- [x] An epic branch whose content was squash-merged into main IS listed in "Epics ready to close" by `apm sync`
- [x] An epic branch whose content was regular-merged into main IS listed in "Epics ready to close" by `apm sync`
- [x] An epic branch whose content has not been merged into main AND all its tickets are terminal IS listed in "Epics ready to submit" (not "close") by `apm sync`
- [x] A new integration test named `sync_empty_epic_behind_main_not_in_close_hints` covers the false-positive scenario: epic with no own commits, main advanced past its starting point, `detect` returns the epic in neither hint list

### Out of scope

- Whether a `done` epic should appear in close hints in the same `apm sync` run that just closed its tickets (the hint is computed before apply, so the tickets appear as `implemented` at hint time)
- Changes to `is_branch_content_merged` itself — the guard is added at the call site in `sync.rs`, not inside the helper
- Changes to `apm epic close`, `apm epic submit`, or `apm epic list`
- Handling of epics with no tickets at all (`derived == "empty"`)

### Approach

#### Change: `apm-core/src/sync.rs`, epic detection block (~line 187)

The `main_ref` variable (already computed above the epic loop, at ~line 43) resolves to `origin/<default>` when the remote ref exists, falling back to the local branch name — reuse it.

Replace the current `is_merged` line:

```rust
let is_merged = git::is_branch_content_merged(root, default_branch, branch)
    .unwrap_or(false);
```

with a two-step guard:

```rust
// An epic branch with no commits beyond its merge-base with main was never
// developed; is_ancestor returns true for such branches (their tip is literally
// reachable from main), producing false positives in epic_close_hints.
let has_own_commits = git::run(root, &["merge-base", &main_ref, branch])
    .ok()
    .and_then(|base| {
        git::run(root, &["rev-list", "--count", &format!("{base}..{branch}")]).ok()
    })
    .and_then(|s| s.trim().parse::<usize>().ok())
    .map(|n| n > 0)
    .unwrap_or(false);

let is_merged = has_own_commits
    && git::is_branch_content_merged(root, default_branch, branch).unwrap_or(false);
```

`git::run` already trims its output, so the SHA from `merge-base` is clean and safe to interpolate into the `rev-list` range.

The `if is_merged` / `else if derived == "done"` block below is unchanged.

#### Test: `apm/tests/integration.rs`

Add `fn sync_empty_epic_behind_main_not_in_close_hints` after the existing `sync_detect_epic_close_hint_after_squash_merge` test. The test:

1. `init_repo()` — standard local repo with no remote
2. Create epic branch with NO commits (branch from main, immediately check out main again) — do NOT use `setup_with_epic` which adds a commit
3. Advance main with an unrelated commit so the epic tip is now an ancestor of main
4. `apm_core::sync::detect(p, &config).unwrap()`
5. Assert epic id is in neither `epic_close_hints` nor `epic_submit_hints`

The two existing tests (`sync_detect_epic_submit_hint`, `sync_detect_epic_close_hint_after_squash_merge`) exercise the non-regressed paths (epic with own commits, not-merged and squash-merged respectively) and should continue to pass unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-03T02:27Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:34Z | groomed | in_design | philippepascal |
| 2026-06-03T06:42Z | in_design | specd | claude |
| 2026-06-03T20:47Z | specd | ready | philippepascal |
| 2026-06-03T20:51Z | ready | in_progress | philippepascal |
| 2026-06-03T20:57Z | in_progress | implemented | claude |
| 2026-06-03T22:22Z | implemented | closed | philippepascal(apm-sync) |
