+++
id = "7cdec85b"
title = "sync misses implemented tickets with post-merge state commits on the ticket branch"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7cdec85b-sync-misses-implemented-tickets-with-pos"
created_at = "2026-05-03T20:15:04.114954Z"
updated_at = "2026-05-04T02:01:08.775726Z"
+++

## Spec

### Problem

`sync`'s `merged_into_main()` uses `git branch --merged <default_branch>` (and a squash-merge variant using `git commit-tree` + `git cherry`) to detect ticket branches that have been merged. Both checks compare the **branch tip** against main. Neither handles the case where a state-transition commit was pushed to the ticket branch *after* the implementation content was already merged into main: the tip is no longer an ancestor of main, and the tip tree differs from what was squash-checked, so both detection paths miss the branch silently.

Observed in ticket 6095305a: implementation merged into main at `2442b358` (via merge commit `bdad99da`), then `f88b9ac0` (state: `merge_failed → implemented`) was committed to the ticket branch. `sync` showed 7 of 8 implemented tickets ready to close; 6095305a was invisible because its branch tip is one commit ahead of the merge point.

Two gaps to close:
1. **Detection**: when neither detection path catches a branch, walk back from the tip skipping commits that touch only files under `tickets/`, find the last real-content commit, and re-run the squash check using that commit's tree. If that tree is in main, the branch content was merged.
2. **Fallback message**: when `sync` cannot determine whether an `implemented` ticket was merged (branch exists but no detection path fires), print a hint directing the user to close it manually.

### Acceptance criteria

- [ ] `sync` detects and offers to close a ticket whose branch has only state-transition commits (touching only files under `config.tickets.dir`) added after the implementation was merged into main via `git merge`
- [ ] `sync` detects and offers to close a ticket whose branch has only state-transition commits added after the implementation was squash-merged into main
- [ ] Tickets detected by the existing `--merged` and squash-merge paths continue to be detected and closed as before (no regression)
- [ ] `sync` prints a hint message for any `implemented` ticket whose branch still exists locally but is not caught by any detection pass; the hint text includes `apm state <id> closed`
- [ ] A ticket branch where non-ticket files were modified after the merge point is not falsely detected as merged by the new pass
- [ ] The hint is not printed for tickets in any state other than `implemented`

### Out of scope

- Preventing the pattern by changing how state commits are written to ticket branches
- Detecting partially-cherry-picked branches (only some implementation commits present in main)
- Branches where the new commits include non-ticket file changes alongside state changes (these are correctly left undetected as the content diverges from what was merged)
- Ticket branches with no implementation content at all (all commits since merge-base are ticket-file-only; return not-merged, nothing to detect)

### Approach

#### New helper: `git_util::content_merged_into_main`

Add `pub fn content_merged_into_main(root: &Path, main_ref: &str, branch: &str, tickets_dir: &str) -> Result<bool>` to `apm-core/src/git_util.rs` (after `squash_merged`).

1. `merge_base` ← `git merge-base <main_ref> <branch>`; on error return `Ok(false)`.
2. `branch_tip` ← `git rev-parse <branch>^{commit}`.
3. If `branch_tip == merge_base` return `Ok(false)` (ancestor; `--merged` handles it).
4. `log_shas` ← `git log --pretty=%H <branch> ^<merge_base>` — SHAs newest-first.
5. Walk from newest SHA: for each sha, `git diff-tree --no-commit-id -r --name-only <sha>`. If any changed path does **not** start with `<tickets_dir>/`, set `content_tip = sha` and stop.
6. If no `content_tip` was set (every commit touched only ticket files) return `Ok(false)`.
7. If `content_tip == branch_tip` return `Ok(false)` — no trailing state commits; `squash_merged` already ran on this and missed it, so the content is not in main.
8. `squash_commit` ← `git commit-tree <content_tip>^{tree} -p <merge_base> -m squash`.
9. `cherry_out` ← `git cherry <main_ref> <squash_commit>`.
10. Return `Ok(cherry_out.trim().starts_with('-'))`.

#### Case 3 in `sync::detect` (`apm-core/src/sync.rs`)

Add `hints: Vec<String>` to the `Candidates` struct (initialize to `Vec::new()` in `detect()`).

Determine `main_ref` at the top of `detect()`: if `refs/remotes/origin/<default>` resolves, use `"origin/<default>"`; otherwise use `"<default>"`. (Mirrors `merged_into_main`'s own preference logic.)

After Case 1 and before Case 2, add Case 3:

- Iterate `branches` not in `merged_set`.
- Call `git::content_merged_into_main(root, &main_ref, branch, &tickets_dir)?`.
- On `true`: read the ticket from the branch (same pattern as Case 1: `read_from_branch` → `Ticket::parse`). If not terminal, push `CloseCandidate { reason: "branch content merged" }`. Insert the branch name into `merged_set` so Case 2 and hint generation don't double-count it.

#### Hint generation in `sync::detect`

After Cases 1–3 (before `Ok(Candidates { ... })`), iterate `branches` still not in `merged_set`:

- `read_from_branch` the ticket; skip parse errors.
- If `state == "implemented"`, push to `candidates.hints`:
  ```
  "ticket #<id> is in `implemented` state but its branch was not detected as merged into \
   main. If it was already merged, close it manually: apm state <id> closed"
  ```

#### Print hints in `apm/src/cmd/sync.rs`

After the `println!("sync: {} ticket branch…")` line and before the close-prompt block, iterate `candidates.hints` and `eprintln!` each one.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-03T20:15Z | — | new | philippepascal |
| 2026-05-04T01:53Z | new | groomed | philippepascal |
| 2026-05-04T01:55Z | groomed | in_design | philippepascal |