+++
id = "25a22125"
title = "apm sync push to origin before scanning tickets. it might make more sense to push after the states have been changed."
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25a22125-apm-sync-push-to-origin-before-scanning-"
created_at = "2026-06-25T00:47:40.559751Z"
updated_at = "2026-06-25T06:55:44.399389Z"
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
- [ ] Push confirmation ("pushed N ahead branches") appears in the terminal after the auto-close messages and ticket branch count line, not before them
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

Add `closed_branches: Vec<String>` to `ApplyOutput`. In `apply`, after each successful `ticket::close` call, derive the branch using the same three-line fallback pattern that `ticket::close` uses internally (at `ticket_util.rs:351–353`):

```rust
let branch = c.ticket.frontmatter.branch.clone()
    .or_else(|| crate::ticket_fmt::branch_name_from_path(&c.ticket.path))
    .unwrap_or_else(|| format!("ticket/{}", c.ticket.frontmatter.id));
```

Push `branch` to `closed_branches`. Re-deriving here (rather than changing `ticket::close`'s return type from `Result<Vec<String>>`) keeps the public API stable and avoids a ripple change through all callers.

#### `apm/src/cmd/sync.rs` — restructure `run` into three sequential blocks

Hoist `ahead_refs: Vec<String>`, `default_is_ahead: bool`, `sync_warnings: Vec<String>`, and `wt_result` as `let mut` bindings above the first `if !offline` block, with empty/false defaults. Then restructure `run` as:

**Block 1 — network I/O and ref reconciliation** (`if !offline`):
Fetch, `sync_non_checked_out_refs` → populate `ahead_refs`, `sync_default_branch` → populate `default_is_ahead`, `sync_checked_out_worktrees` → populate `wt_result`, collect worktree warnings into `sync_warnings`. No push in this block.

**Block 2 — detect and apply** (unconditional, runs in offline mode too — exactly as today):
`sync::detect` → `candidates`, print ticket branch count, print hints, prompt/auto-close, `sync::apply` → `apply_out`, merge `apply_out.closed_branches` into `ahead_refs` (iterate `closed_branches`, push each entry not already in `ahead_refs`), print apply messages, print epic hints.

**Block 3 — push and output** (`if !offline`):
1. If `default_is_ahead`: prompt or auto-push. On push: call `sync_warnings.retain(|w| !w.contains(&config.project.default_branch) || !w.contains("ahead"))` to drop the MAIN_AHEAD warning, then push and print confirmation.
2. If `!ahead_refs.is_empty()`: prompt or auto-push (`--push-refs`). Push each branch in `ahead_refs` (which now includes branches that became ahead during block 2). Print confirmation.
3. Print `sync_warnings` (MAIN_AHEAD already removed if push happened in step 1).
4. Print worktree summary.

Moving the warnings print and worktree summary from their current position (inside the original `if !offline` block, before detect+apply) to block 3 (after push) ensures the `retain` call takes effect before warnings are emitted. The contradictory "stay in place" instruction from the previous Approach is removed.

#### Output ordering change

After the reorder, a typical sync run produces output in this sequence:

1. Worktree fast-forward lines (block 1)
2. `sync: N ticket branches visible` (block 2)
3. Close messages and hints (block 2)
4. Epic hints (block 2)
5. Push prompts / push confirmations (block 3)
6. Warnings and worktree summary (block 3)

This is intentional. Previously, push prompts appeared between fast-forward lines and the ticket count. The new order is strictly more useful: the push covers the full set of ahead branches including those just closed.

#### Test

Add an integration test in `apm/tests/integration.rs` alongside the existing `sync_closes_*` tests:

1. Set up a bare origin repo and clone it into a working tree.
2. Create a ticket branch at origin with the ticket in `implemented` state; fast-forward the local ref so the branch is Equal (not ahead).
3. Merge the ticket branch into `main` at origin (simulating a merged PR).
4. Fetch origin so local sees the merge.
5. Call `apm::cmd::sync::run(root, offline=false, quiet=true, no_aggressive=true, auto_close=true, push_default=false, push_refs=true)`.
6. Assert that `git log origin/<ticket-branch>` includes a commit whose message contains `"close"` — confirming the close commit written by `apply` was pushed to origin within the same sync run.

### Open questions


### Amendment requests

- [x] Handle the offline/non-offline boundary in the reorder. detect+apply currently run OUTSIDE the 'if !offline' block (apm/src/cmd/sync.rs:132+) and must keep running in offline mode (auto-close + hints, no push). The Approach's linear 9-step list hides this. Specify: hoist ahead_refs/default_is_ahead out of the offline block, run detect+apply unconditionally, then push inside a second 'if !offline' block that consumes both ahead_refs and apply_out.closed_branches.
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
| 2026-06-25T06:55Z | amend | in_design | philippepascal |