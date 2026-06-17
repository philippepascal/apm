+++
id = "b9aed386"
title = "Propagate terminal state to main for already-merged epic tickets"
state = "in_progress"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b9aed386-propagate-terminal-state-to-main-for-alr"
created_at = "2026-06-17T00:19:57.975332Z"
updated_at = "2026-06-17T00:42:19.398065Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, its terminal-state transitions (close, and other terminal states) are committed to two places: the ticket own branch (authoritative, hard write) and target_branch which is the epic branch (best-effort, soft write). See close() at apm-core/src/ticket/ticket_util.rs:355-362 and transition() at apm-core/src/state.rs:178-193. After the epic code is merged to main via apm epic submit, any later terminal transitions (for example apm sync closing implemented tickets) append ticket-file-only commits to the epic branch but never to main. For an epic ticket target_branch is the epic branch, so close() never writes the closed state to main (contrast: a non-epic ticket has target_branch None so it defaults to main and closes directly onto main).

Consequence: a closed epic ticket has its final state only on the ticket branch and the epic branch. The ticket file on main stays frozen at the epic-submit-time state (for example implemented). The only mechanism that reconciles this to main is the manual apm archive command, which reads the ticket branch (apm-core/src/archive.rs:71) and writes the closed content into the archive directory on main (archive.rs:143). That reconciliation is manual, separate, and order-dependent: it only works while the ticket branch still exists. If apm clean-branches prunes the ticket branch before apm archive runs, the closed state is lost from everywhere reachable and main shows the stale state permanently. apm epic close deleting the epic branch makes this easier to hit because closing the epic becomes frictionless.

Proposed root-cause fix (supersedes ticket 57423ff5, which only relaxed the apm epic close guard and treated the symptom): when a ticket target_branch is an epic that is already merged to main, route the terminal-state commit to the default branch (main) instead of, or in addition to, the dead epic branch. Reuse the existing detection git_util::content_merged_into_main(root, main_ref, branch, tickets_dir) to decide already-merged. This is proven machinery: non-epic tickets already close straight to main via target = target_branch.unwrap_or(default_branch) at ticket_util.rs:358. Apply the choice in both close() (ticket_util.rs:358-362) and the transition() terminal path (state.rs:187-193).

Why this over the alternative considered (apm sync doing an internal apm epic submit): epic submit defaults to opening a PR and only merges with --merge or --auto, requires the main worktree to be checked out on the default branch (it bails otherwise), creates a fresh merge commit on main for every sync that touches an epic ticket, and inherits the whole merge-failure surface inside what should be a safe refresh. The transition-layer write is a small single-file commit that works in PR and merge workflows, has no conflict surface, needs no particular checkout, and covers manual apm state as well as sync.

Acceptance should include an integration test proving that closing an epic ticket whose epic code is already merged lands the closed state on main, and that apm epic close then succeeds under the original is_branch_content_merged guard. Consider whether the redundant epic-branch write should be dropped entirely once the epic is merged. Out of scope: changing apm epic submit; changing apm archive.

### Acceptance criteria

- [ ] `apm state <id> closed` on a ticket whose `target_branch` is an epic branch already merged to the default branch writes `state = "closed"` to the default branch, not to the epic branch
- [ ] `apm close <id>` on the same ticket also writes `state = "closed"` to the default branch
- [ ] When the ticket's `target_branch` epic branch has not yet been merged, both commands continue to commit the terminal state to `target_branch` (existing behavior unchanged)
- [ ] `apm epic close <epic-id>` succeeds without `--force` for a merged, fully-closed epic whose tickets were closed via the new routing (no regression in the `is_branch_content_merged` guard or the non-terminal-tickets check)
- [ ] An integration test covers the no-trailing-commits case: create epic ticket → regular merge (--no-ff) epic branch to main → `ticket::close` → assert default branch has `state = "closed"`
- [ ] An integration test covers the trailing-commits case: same setup, then add a ticket-state-only commit to the epic branch after the merge → `ticket::close` → assert default branch has `state = "closed"`
- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Changing `apm epic submit`
- Changing `apm archive` or the order-dependent reconciliation it provides
- Changing `apm sync`'s detection logic for finding close candidates (Cases 1–4 in `sync::detect`)
- Changing `apm clean-branches` behavior
- Adding auto-archiving or any other reconciliation beyond the secondary write

### Approach

At the two call sites that perform the secondary write (to `target_branch` or the default branch), replace the naive `target_branch.unwrap_or(default_branch)` resolution with a merged-epic check. When the resolved `target_branch` is detected as already merged to the default branch, route the write to the default branch instead and skip the write to the epic branch.

#### Merged-epic detection

Two functions are needed in combination because they cover complementary cases:

- `git::is_branch_content_merged(root, default_branch, target_branch)` — catches the case where the epic tip has **no trailing ticket-state commits after the merge** (epic tip is a git ancestor of main; `is_ancestor` returns true immediately)
- `git::content_merged_into_main(root, main_ref, target_branch, tickets_dir)` — catches both regular-merge and squash-merge cases where the epic branch has **trailing ticket-state commits added after the code was merged** (epic tip is no longer an ancestor of main, but the last non-ticket commit on the epic IS in main)

Neither function alone covers all combinations of (regular/squash merge) × (with/without trailing state commits). Use them OR'd; short-circuit on the first true result.

Compute `main_ref` preferring `origin/<default_branch>` when available, matching the pattern used in `sync::detect`:

```rust
let remote_ref = format!("refs/remotes/origin/{default}");
let main_ref = if crate::git::run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
    format!("origin/{default}")
} else {
    default.to_string()
};
```

The secondary write is **replaced, not supplemented**: when the epic is merged, write only to the default branch. There is no value in also writing to an epic branch that is about to be deleted by `apm epic close`, and doing so creates a spurious commit on a dead ref.

#### `apm-core/src/ticket/ticket_util.rs` — `close()` (lines 358–362)

Declare `default`, `tickets_dir`, and `main_ref` before the existing target computation, then replace the two-liner with:

```rust
let effective_target: String = match t.frontmatter.target_branch.as_deref() {
    Some(tb) => {
        let already_merged =
            crate::git::is_branch_content_merged(root, default, tb).unwrap_or(false)
            || crate::git::content_merged_into_main(root, &main_ref, tb, &tickets_dir)
                .unwrap_or(false);
        if already_merged { default.to_string() } else { tb.to_string() }
    }
    None => default.to_string(),
};
if let Err(e) = crate::git::commit_to_branch(root, &effective_target, &rel_path, &content,
    &format!("ticket({id}): close"))
{
    output.push(format!("warning: commit closed state to {effective_target} failed: {e:#}"));
}
```

`default` is `config.project.default_branch.as_str()` (borrow it first); `tickets_dir` is `config.tickets.dir.to_string_lossy().into_owned()`.

#### `apm-core/src/state.rs` — `transition()` (lines 187–193)

Apply the same replacement inside the `if target_is_terminal { … }` block. All required variables (`root`, `config`, `t`) are in scope. Do not extract a shared helper function — the duplication is six lines in two files within the same crate and is clear in context.

#### Integration tests in `apm/tests/integration.rs`

Add three tests near the existing `close_epic_scoped_writes_to_epic_not_main` test, using the existing `init_repo()`, `setup_with_epic()`, `git()`, `branch_content()`, `ticket_rel_path()`, and `apm_core::git::commit_to_branch` helpers. Each test sets up an epic ticket manually via `commit_to_branch` (same pattern as `close_epic_scoped_writes_to_epic_not_main`) to avoid the CLI dependency.

**`close_merged_epic_writes_to_main`**
1. `setup_with_epic()` — creates `epic/<id>-my-epic` branch with one commit
2. Write a ticket file with `target_branch = epic/...` to both the ticket branch and the epic branch via `commit_to_branch`
3. `git merge --no-ff epic/...` — regular merge to main (no trailing commits on epic)
4. `apm_core::ticket::close(p, &config, ticket_id, None, "test", false)`
5. Assert `branch_content(p, "main", &rel)` contains `state = "closed"`
6. Assert `branch_content(p, epic_branch, &rel)` does NOT contain `state = "closed"` (the write was redirected; no double-write to the epic branch)
7. Assert `apm::cmd::epic::run_close(p, &epic_id, false)` succeeds (covers AC 4)

**`close_merged_epic_trailing_commits_writes_to_main`**
Same setup as above, but after the `--no-ff` merge add a ticket-state-only commit to the epic branch (e.g., commit a dummy `tickets/other.md` to the epic branch to simulate a prior state write). Then call `ticket::close` on the original ticket and assert default branch has `state = "closed"`. This exercises the `content_merged_into_main` detection path.

**`state_transition_closed_merged_epic_writes_to_main`**
Same setup as the first test (merge epic to main, no trailing commits), but call `apm_core::state::transition(p, &ticket_id, "closed".into(), true, false)` instead of `ticket::close`, and assert default branch has `state = "closed"`.

The existing `close_epic_scoped_writes_to_epic_not_main` test (unmerged epic) must continue to pass unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-17T00:19Z | — | new | philippepascal |
| 2026-06-17T00:21Z | new | groomed | philippepascal |
| 2026-06-17T00:21Z | groomed | in_design | philippepascal |
| 2026-06-17T00:31Z | in_design | specd | claude |
| 2026-06-17T00:42Z | specd | ready | philippepascal |
| 2026-06-17T00:42Z | ready | in_progress | philippepascal |