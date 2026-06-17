+++
id = "57423ff5"
title = "refine apm epic close"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/57423ff5-refine-apm-epic-close"
created_at = "2026-06-16T20:16:27.782398Z"
updated_at = "2026-06-17T00:18:58.151881Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, `target_branch` in its frontmatter is set to the epic branch. `ticket::close()` commits the state change to both the ticket's own branch and to `target_branch`. After the epic's code is merged to `main` via `apm epic submit`, subsequent close operations (e.g. `apm sync` closing implemented tickets) append ticket-state commits to the epic branch on top of the already-merged code. The epic branch tip is no longer an ancestor of `main`, so `is_branch_content_merged` returns false and `apm epic close` refuses with "epic has N commit(s) not yet in main."

The problem is specific to the `apm epic close` guard and the `apm sync` epic-close-hint detection: both use `is_branch_content_merged`, which performs a simple ancestor / squash-cherry check against the branch tip. That check does not account for trailing ticket-file-only commits. The function `content_merged_into_main` (used for ticket branches in sync Case 3) already handles this correctly — it walks back from the branch tip, strips ticket-file-only commits, and checks if the resulting "content tip" is in `main`. Epic branches need the same treatment.

### Acceptance criteria

- [ ] `apm epic close <id>` succeeds (exit 0, deletes local and remote epic branches) when the epic's code has been merged to `main` and the only unmerged commits on the epic branch are ticket-file-only state-transition commits
- [ ] `apm epic close <id>` still fails with "epic has N commit(s) not yet in main" when the epic has genuine unmerged code commits (files outside `tickets/`)
- [ ] `apm sync` shows the epic-close hint for an epic that meets the same condition (merged code, trailing ticket-only commits)
- [ ] `apm epic close <id> --force` continues to delete the epic branch unconditionally regardless of commit content (no regression)
- [ ] `cargo test --workspace` passes, including all pre-existing `is_branch_content_merged` and `content_merged_into_main` tests
- [ ] A new integration test covers: epic merged to `main` via `--no-ff`, state-transition commits appended, `run_close` returns `Ok`
- [ ] A new integration test covers: same scenario with squash merge of the epic to `main`

### Out of scope

- Changing `ticket::close()` to redirect state commits away from the epic branch when the epic is already merged — that would fix the problem at the source but is a separate, larger change
- Auto-merging ticket-state commits to `main` as part of `apm epic close` — the `archive` command already handles this by reading from the ticket branch directly, so merging is unnecessary
- Changing `apm epic submit` behavior
- Handling epics with `blocked` or `question` tickets (separate quiescence guard)

### Approach

The existing function `git_util::content_merged_into_main(root, main_ref, branch, tickets_dir)` already does exactly what is needed: it walks commits from the branch tip newest-first, skips trailing ticket-file-only commits, and checks whether the "content tip" (last non-ticket commit) is present in `main` via squash-cherry or first-parent detection. The fix wires this function in as a fallback after `is_branch_content_merged` in the two affected call sites.

#### `apm/src/cmd/epic.rs` — `run_close`, step 4

Current (line ~247):
```rust
let is_merged = apm_core::git::is_branch_content_merged(root, default_branch, &epic_branch)?;
```

Replace with:
```rust
let tickets_dir = config.tickets.dir.to_string_lossy();
let is_merged = apm_core::git::is_branch_content_merged(root, default_branch, &epic_branch)?
    || apm_core::git::content_merged_into_main(root, &main_ref, &epic_branch, &tickets_dir)?;
```

`main_ref` is already computed in step 3 (prefers `origin/<default>` when available). `config.tickets.dir` is available via `CmdContext::load_config_only`. No other logic in `run_close` changes.

#### `apm-core/src/sync.rs` — `detect`, epic detection pass (line ~203)

Current:
```rust
let is_merged = has_own_commits
    && git::is_branch_content_merged(root, default_branch, branch).unwrap_or(false);
```

Replace with:
```rust
let is_merged = has_own_commits
    && (git::is_branch_content_merged(root, default_branch, branch).unwrap_or(false)
        || git::content_merged_into_main(root, &main_ref, branch, &tickets_dir).unwrap_or(false));
```

`main_ref` and `tickets_dir` are already computed earlier in `detect`.

#### Tests

Add two integration tests to `apm/tests/integration.rs`, modeled on the existing `epic_close_succeeds_on_regular_merged_branch` and `epic_close_succeeds_on_squash_merged_branch` tests but with a ticket-file-only commit appended to the epic branch after the merge:

- `epic_close_allows_ticket_state_commits_after_noff_merge`: Call `setup_epic_with_commit()`, no-ff merge the epic branch into `main`, check out the epic branch, add a commit touching only a file under `tickets/` (simulating a ticket-state transition), check out `main`, then call `run_close(p, &epic_id, false)` and assert it returns `Ok`. Assert the epic branch is deleted afterward.

- `epic_close_allows_ticket_state_commits_after_squash_merge`: Same structure but squash-merge the epic (`git merge --squash` + commit), then append the ticket-only commit before calling `run_close`.

Both tests exercise the `|| content_merged_into_main(...)` fallback wiring in `run_close` directly. `content_merged_into_main` is already tested in isolation at `git_util.rs:2341` and `:2370` for these merge modes; these integration tests verify the call-site wiring.

### Open questions


### Amendment requests

- [x] The test plan in the Approach contradicts ACs #6/#7 and leaves the new wiring uncovered. ACs #6/#7 require integration tests where the epic is merged (no-ff / squash) with appended state commits and run_close returns Ok. But the Approach instead adds inline epic.rs tests asserting on content_merged_into_main directly. That is (a) redundant — content_merged_into_main is already tested for these exact cases at git_util.rs:2341 (content_merged_into_main_regular_merge_with_state_commit) and :2370 (..._squash_merge_with_state_commit) — and (b) does not exercise the actual change, which is the '|| content_merged_into_main(...)' fallback wiring in run_close (epic.rs:247) and sync::detect (sync.rs:203). Replace the test plan with an integration test that drives run_close / 'apm epic close' through the new fallback for the merged-code + trailing-ticket-only-commits scenario (both no-ff and squash), asserting it now succeeds and deletes the branch. Model it on the existing epic-close integration test epic_close_blocks_on_implemented_state in apm/tests/integration.rs. This both satisfies ACs #6/#7 and actually covers the new wiring.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T20:16Z | — | new | philippepascal |
| 2026-06-16T22:35Z | new | groomed | philippepascal |
| 2026-06-16T22:35Z | groomed | in_design | philippepascal |
| 2026-06-16T22:43Z | in_design | specd | claude |
| 2026-06-16T23:10Z | specd | amend | philippepascal |
| 2026-06-16T23:11Z | amend | in_design | philippepascal |
| 2026-06-16T23:13Z | in_design | specd | claude |
| 2026-06-16T23:51Z | specd | ready | philippepascal |
| 2026-06-17T00:18Z | ready | closed | philippepascal |
