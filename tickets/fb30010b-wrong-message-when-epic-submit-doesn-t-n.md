+++
id = "fb30010b"
title = "wrong message when epic submit doesn't need to merge anything"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fb30010b-wrong-message-when-epic-submit-doesn-t-n"
created_at = "2026-09-01T00:59:43.049321Z"
updated_at = "2026-09-01T01:44:48.408324Z"
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

- [x] `apm epic submit <id>` (default `--pr` behaviour) on an epic branch with zero commits ahead of the default branch prints a "nothing to submit" message and exits 0, without invoking `gh pr create`
- [x] `apm epic submit <id> --merge` on an epic branch with zero commits ahead of the default branch prints the same "nothing to submit" message and exits 0, without attempting `git merge` and without any "merge conflict" text
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
bindings at lines 88-90) and before the existing "Determine whether to merge
locally or push+PR" block (line 92). The guard runs unconditionally, before
`do_merge` is computed, so it covers `--pr` (default), `--merge`, and
`--auto` in one place without per-mode branching.

Count commits reachable from `epic_branch` but not `default_branch` using
`std::process::Command` directly — `apm_core::git_util::run` is `pub(crate)`
inside `apm-core` (`apm-core/src/git_util.rs:7`) and is not visible from the
`apm` crate. Mirror the existing rev-list pattern already used in
`run_close` (`apm/src/cmd/epic.rs:251-255`):

```rust
let count_out = std::process::Command::new("git")
    .current_dir(root)
    .args(["rev-list", "--count", &format!("{default_branch}..{epic_branch}")])
    .output()?;
let ahead = String::from_utf8_lossy(&count_out.stdout).trim().parse::<u64>().unwrap_or(0);
if ahead == 0 {
    println!("epic has no commits beyond {default_branch}; nothing to submit");
    println!("if this epic's work already landed, run `apm epic close {epic_id}`");
    return Ok(());
}
```

Do not source this `ahead` value from `apm_core::epic::merge_tree_status`
(already called at line 94 for `auto_mode`'s clean check) — its `ahead`
field counts `{epic_branch}..{default_branch}` (commits `default_branch` has
that `epic_branch` doesn't; the direction `run_refresh_epic` needs to decide
whether the epic needs refreshing from main). Submit needs the opposite
direction — commits `epic_branch` has that `default_branch` doesn't — so
this must be a separate `rev-list --count` call in the
`default_branch..epic_branch` order as above; reusing `merge_tree_status`'s
`ahead` here would silently check the wrong direction.

This early-return mirrors the existing "print and return Ok(())" pattern
`run_refresh_epic` already uses for its analogous "epic branch is up to date
with {default_branch}" case (`apm/src/cmd/epic.rs:353-356`).

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
`setup_epic_with_commit` helper and the `run_apm` subprocess helper
(`apm/tests/integration.rs:16`). The three "nothing to submit" tests must go
through `run_apm`, not a direct `run_submit` call: `println!` output from an
in-process call is not visible to the test's own assertions, only
`run_apm`'s subprocess `Output.stdout` is capturable.

- No-ff merge the epic branch fully into main first (so `ahead == 0`, as in
  `epic_close_succeeds_on_regular_merged_branch`'s setup at line 9621-9622),
  then call `run_apm(p, &["epic", "submit", &epic_id])` (default `--pr`
  path) and assert the returned `Output.stdout` contains "nothing to submit"
  and "apm epic close". `run_apm` already asserts `status.success()`
  internally, so a successful return is itself the exit-0 assertion.
- The same setup, but `run_apm(p, &["epic", "submit", &epic_id, "--merge"])`,
  asserting the same stdout message and that neither stdout nor stderr
  contains "merge conflict".
- The same setup with `run_apm(p, &["epic", "submit", &epic_id, "--auto"])`,
  asserting the same short-circuit message.
- A regression test that constructs a genuine conflict (epic and main both
  modify the same file differently, as in
  `merge_ref_conflict_aborts_and_warns` in `apm-core/src/git_util.rs:2183`)
  and calls `apm::cmd::epic::run_submit` directly with `merge=true` — unlike
  the three tests above, this one only needs to assert on the returned `Err`
  string, not on stdout, so it can keep the in-process call style already
  used by `epic_submit_merge_then_close` — asserting the error is unchanged:
  `merge conflict — resolve manually ...`.

### Open questions


### Amendment requests

- [x] Approach: apm_core::git_util::run is pub(crate) (apm-core/src/git_util.rs:7) and cannot be called from the apm crate. Replace the rev-list snippet with a std::process::Command call, mirroring the existing rev-list --count in run_close (apm/src/cmd/epic.rs:251-255), or state that git_util::run must be made pub.
- [x] Tests: the three 'nothing to submit' tests cannot assert on stdout by calling run_submit directly, since println! output is not capturable. Drive them through the run_apm helper (apm/tests/integration.rs:16) with args epic submit <id> [--merge|--auto] and assert on the returned Output stdout and exit status. The conflict regression test may still call run_submit directly since it asserts on the returned error.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-09-01T00:59Z | — | new | philippepascal |
| 2026-09-01T01:04Z | new | groomed | philippepascal |
| 2026-09-01T01:04Z | groomed | in_design | philippepascal |
| 2026-09-01T01:07Z | in_design | specd | claude |
| 2026-09-01T01:11Z | specd | amend | philippepascal |
| 2026-09-01T01:34Z | amend | in_design | philippepascal |
| 2026-09-01T01:36Z | in_design | specd | claude |
| 2026-09-01T01:44Z | specd | ready | philippepascal |
| 2026-09-01T01:44Z | ready | in_progress | philippepascal |