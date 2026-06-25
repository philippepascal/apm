+++
id = "25a22125"
title = "apm sync push to origin before scanning tickets. it might make more sense to push after the states have been changed."
state = "amend"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25a22125-apm-sync-push-to-origin-before-scanning-"
created_at = "2026-06-25T00:47:40.559751Z"
updated_at = "2026-06-25T06:55:00.347063Z"
+++

## Spec

### Problem

`apm sync` currently pushes locally-ahead ticket and epic branches to origin **before** it scans for tickets to auto-close. This means any close commits written by `sync::apply` (via `ticket::close`) are left sitting on local branches — they are not published to origin within the same sync run and only reach origin on the next `apm sync` invocation.

The correct order is: detect merge candidates, apply closures, then push. With that ordering, the push prompt covers every pending local commit in one shot — including close-state commits just written by the auto-close step — so origin stays current after a single `apm sync`. The reordering also applies to the default-branch push, which belongs at the end for the same reason.

### Acceptance criteria

- [ ] `apm sync` (non-offline) runs `sync::detect` and `sync::apply` before prompting to push ahead branches to origin
- [ ] When `sync::apply` closes a ticket whose branch was Equal to origin at the start of the run, the resulting ahead branch is included in the push prompt in the same sync invocation
- [ ] `apm sync --push-refs` (non-interactive push) pushes all ahead branches after auto-close, including branches that only became ahead due to the closure
- [ ] The default-branch push prompt appears after the auto-close step, not before
- [ ] Quiet mode (`--quiet`) suppresses push output the same as before
- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Changes to fetch behavior or the fast-forward logic in `sync_non_checked_out_refs`
- Changes to worktree reconciliation (`sync_checked_out_worktrees`)
- Push behavior for `apm state` transitions (those push immediately at transition time)
- The `--auto-close` flag semantics or the close-candidate detection logic
- Offline mode (`--offline`), which skips all network I/O including the push step

### Approach

Two files change.

#### `apm-core/src/sync.rs` — expose closed branches from apply

Add `closed_branches: Vec<String>` to `ApplyOutput`. In `apply`, after each successful `ticket::close` call, extract the branch from `c.ticket.frontmatter.branch` (with the same fallback logic `ticket::close` itself uses: `branch_name_from_path` → `ticket/{id}`) and push it to `closed_branches`. This gives the caller visibility into which branches received a close commit.

#### `apm/src/cmd/sync.rs` — move push block to after detect+apply

Current execution order in `run`:
1. Fetch
2. `sync_non_checked_out_refs` → `ahead_refs`
3. `sync_default_branch` → `default_is_ahead`
4. `sync_checked_out_worktrees`
5. **Push default branch** (if `default_is_ahead`)
6. **Push `ahead_refs`**
7. `sync::detect` → `candidates`
8. `sync::apply` → `apply_out`

New order:
1. Fetch
2. `sync_non_checked_out_refs` → `ahead_refs`
3. `sync_default_branch` → `default_is_ahead`
4. `sync_checked_out_worktrees`
5. `sync::detect` → `candidates`
6. `sync::apply` → `apply_out`
7. Merge `apply_out.closed_branches` into `ahead_refs` (dedup via `HashSet<String>`)
8. **Push default branch** (if `default_is_ahead`)
9. **Push `ahead_refs`** (now includes branches that became ahead due to step 6)

Steps 8–9 are the existing push blocks relocated after step 6, with one extra dedup step (7) before them. No changes to prompt wording, `--push-refs` flag handling, or `--quiet` behaviour; `sync_warnings` collection and the worktree summary print stay in place.

#### Test

Add an integration test in `apm/tests/integration.rs` alongside the existing `sync_closes_*` tests:

1. Set up a bare origin repo and clone it into a working tree.
2. Create a ticket branch at origin with the ticket in `implemented` state, and fast-forward the local ref so the branch is Equal (not ahead).
3. Merge the ticket branch into `main` at origin (simulating a merged PR).
4. Fetch origin so local sees the merge.
5. Call `apm::cmd::sync::run(root, offline=false, quiet=true, no_aggressive=true, auto_close=true, push_default=false, push_refs=true)`.
6. Assert that `git log origin/<ticket-branch>` includes a commit whose message contains `"close"` — confirming the close commit written by `apply` was pushed to origin within the same sync run.

### Open questions


### Amendment requests

- [ ] Handle the offline/non-offline boundary in the reorder. detect+apply currently run OUTSIDE the 'if !offline' block (apm/src/cmd/sync.rs:132+) and must keep running in offline mode (auto-close + hints, no push). The Approach's linear 9-step list hides this. Specify: hoist ahead_refs/default_is_ahead out of the offline block, run detect+apply unconditionally, then push inside a second 'if !offline' block that consumes both ahead_refs and apply_out.closed_branches.
- [ ] Fix the sync_warnings print-ordering conflict. The default-branch push calls sync_warnings.retain(...) at sync.rs:74 to drop the MAIN_AHEAD warning BEFORE warnings print at :102. Moving push below detect+apply while keeping the warnings print and worktree summary 'in place' (as the current Approach states) would print MAIN_AHEAD even when the user chose to push — a regression. The warnings print and worktree summary must move below the relocated push (or the retain logic restructured). Remove the contradictory 'stay in place' instruction from the Approach.
- [ ] Document the visible output-ordering change: after the reorder, 'pushed N ahead branches' prints after 'sync: N ticket branches visible' and the close messages. Add a one-line note (or an AC) so this is expected and not later flagged as a regression.
- [ ] Optional/minor: avoid duplicating the branch-fallback logic in apply. ticket::close already computes the branch (frontmatter.branch -> branch_name_from_path -> ticket/{id}) at ticket_util.rs:351-353; consider returning it from close instead of re-deriving in apply to keep the fallback in one place. Acceptable to keep re-derivation if simpler.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-25T00:47Z | — | new | philippepascal |
| 2026-06-25T06:41Z | new | groomed | philippepascal |
| 2026-06-25T06:41Z | groomed | in_design | philippepascal |
| 2026-06-25T06:48Z | in_design | specd | claude |
| 2026-06-25T06:55Z | specd | amend | philippepascal |