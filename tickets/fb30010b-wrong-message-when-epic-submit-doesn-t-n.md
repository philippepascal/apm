+++
id = "fb30010b"
title = "wrong message when epic submit doesn't need to merge anything"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fb30010b-wrong-message-when-epic-submit-doesn-t-n"
created_at = "2026-09-01T00:59:43.049321Z"
updated_at = "2026-09-01T01:07:36.682221Z"
+++

## Spec

### Problem

`apm epic submit` produces misleading, actionable-free errors when the epic
branch has no commits beyond the default branch — e.g. because its tickets
already landed via another path, or the branch was already merged manually.

With the default `--pr` mode, `run_submit` (`apm/src/cmd/epic.rs`) shells out
straight to `gh pr create`. When there is nothing to submit, `gh` fails with
a raw GraphQL error that gets surfaced verbatim: `Error: gh pr create failed:
... GraphQL: No commits between main and epic/... (createPullRequest)`. With
`--merge`, `apm_core::git_util::merge_ref` treats "Already up to date" (a
successful no-op `git merge`) identically to a real conflict — both paths
return `None` — so `run_submit` reports `Error: merge conflict — resolve
manually after checking out main, or use --pr to open a PR instead`, even
though there is no conflict: `git merge` itself would report "Already up to
date."

Both messages send the user chasing a nonexistent problem instead of telling
them the real situation: there is nothing new to submit, and the epic branch
can most likely just be closed with `apm epic close`.

### Acceptance criteria

- [ ] `apm epic submit <id>` (default `--pr` behaviour) on an epic branch with zero commits ahead of the default branch prints a "nothing to submit" message and exits 0, without invoking `gh pr create`
- [ ] `apm epic submit <id> --merge` on an epic branch with zero commits ahead of the default branch prints the same "nothing to submit" message and exits 0, without attempting `git merge` and without any "merge conflict" text
- [ ] `apm epic submit <id> --auto` on an epic branch with zero commits ahead of the default branch also short-circuits with the "nothing to submit" message and exits 0, without attempting a merge or a PR
- [ ] The "nothing to submit" message names the default branch and suggests `apm epic close <id>` as the next step
- [ ] `apm epic submit <id> --merge` on an epic branch that has unmerged commits and a genuine merge conflict still reports `merge conflict — resolve manually ...` unchanged
- [ ] `apm epic submit <id> --merge` on an epic branch with unmerged, cleanly-mergeable commits still merges into the default branch and reports success unchanged
- [ ] `apm epic submit <id>` (default `--pr`) on an epic branch with unmerged commits still opens or updates a PR unchanged

### Out of scope

- Fixing the general conflation in `apm_core::git_util::merge_ref`, where a `None` return means either "already up to date" or "real conflict" — this ticket only needs `apm epic submit` to stop reaching that call at all when there's nothing to merge. Other callers (`apm-core/src/start.rs`, `apm epic refresh` in `apm/src/cmd/epic.rs`) are unaffected: `run_refresh_epic` already guards this exact case with its own upfront `ahead == 0` check before calling `merge_ref`.
- Adding quiescence or live-worker checks to `apm epic submit` — only `apm epic close` has those today.
- Any change to `apm epic close` behaviour.
- Auto-closing the epic when there is nothing left to submit — the fix only suggests `apm epic close <id>` as the next step, it does not run it.

### Approach

In `apm/src/cmd/epic.rs::run_submit`, add an upfront guard right after the
epic branch is resolved (after the `epic_id`/`pr_title`/`default_branch`
bindings around line 88-90) and before the existing "Determine whether to
merge locally or push+PR" block (line 92 today). The guard runs unconditionally,
before `do_merge` is computed, so it covers `--pr` (default), `--merge`, and
`--auto` in one place without per-mode branching.

Count commits reachable from `epic_branch` but not `default_branch`:

```rust
let ahead = apm_core::git_util::run(
    root,
    &["rev-list", "--count", &format!("{default_branch}..{epic_branch}")],
)?
.trim()
.parse::<usize>()
.unwrap_or(0);
if ahead == 0 {
    println!("epic has no commits beyond {default_branch}; nothing to submit");
    println!("if this epic's work already landed, run `apm epic close {epic_id}`");
    return Ok(());
}
```

This mirrors the existing `rev-list --count base..branch` pattern already
used in `apm-core/src/git_util.rs:928` and `apm-core/src/sync.rs:198`, and
the existing "print and return Ok(())" early-exit pattern `run_refresh_epic`
already uses for its analogous "epic branch is up to date with
{default_branch}" case (`apm/src/cmd/epic.rs:353-356`).

With this guard in place, the two reported symptoms both disappear at the
source: the `--pr` path never reaches `gh_pr_create_or_update` (so `gh pr
create`'s raw "No commits between..." GraphQL failure is never surfaced),
and the `--merge` path never reaches `apm_core::git_util::merge_ref` (so the
"Already up to date" no-op merge can no longer be misreported as "merge
conflict"). No changes are needed to `merge_ref`, `gh_pr_create_or_update`,
or `merge_tree_status` — the existing conflict-handling code after the guard
(lines 98-156) is untouched and still applies whenever there genuinely are
unmerged commits.

Add integration tests in `apm/tests/integration.rs` near
`epic_submit_merge_then_close` (~line 9823), reusing the existing
`setup_epic_with_commit` / `setup_with_epic` helpers:

- A test that merges the epic branch fully into main first (so `ahead == 0`,
  as in `epic_close_succeeds_on_regular_merged_branch`'s setup at line
  9621-9622), then calls `run_submit` with `merge=false, pr=false,
  auto_mode=false` (default `--pr` path) and asserts `Ok(())` plus stdout
  containing "nothing to submit" and "apm epic close".
- The same setup, but calling `run_submit` with `merge=true` and asserting
  the same message and `Ok(())`, with no "merge conflict" text anywhere in
  the error/output.
- The same setup with `auto_mode=true`, asserting the same short-circuit.
- A regression test that constructs a genuine conflict (epic and main both
  modify the same file differently, as in
  `merge_ref_conflict_aborts_and_warns` in `apm-core/src/git_util.rs:2183`)
  and asserts `run_submit` with `merge=true` still returns the unchanged
  `merge conflict — resolve manually ...` error.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-09-01T00:59Z | — | new | philippepascal |
| 2026-09-01T01:04Z | new | groomed | philippepascal |
| 2026-09-01T01:04Z | groomed | in_design | philippepascal |