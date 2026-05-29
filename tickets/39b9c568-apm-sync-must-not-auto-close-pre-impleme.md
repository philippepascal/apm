+++
id = "39b9c568"
title = "apm sync must not auto-close pre-implementation tickets"
state = "implemented"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/39b9c568-apm-sync-must-not-auto-close-pre-impleme"
created_at = "2026-05-29T00:18:15.128911Z"
updated_at = "2026-05-29T00:41:18.006760Z"
+++

## Spec

### Problem

BUG: apm sync auto-closes tickets in pre-implementation states (new/groomed/specd/question) when their branch's fork point reached main via an unrelated merge. Concretely: a side-note ticket (apm new --side-note creates a plain ticket in 'new' state whose branch only ever contains the ticket .md file, no implementation, no frontmatter marker) gets flagged as 'branch content merged' and closed by sync.

ROOT CAUSE: sync::detect Case 3 calls git_util::content_merged_into_main. For a branch with zero non-ticket-file commits, content_tip is None, and step 6's regular-merge sub-case fires: it sees merge_base is not on main's first-parent chain (because the branch's fork point was pulled into main via an epic's --no-ff merge commit's side parent) and returns true. But the branch's own commits were never merged into main. For a real ticket the implementation commit sits BELOW the merge-base (so it is in main); for an unmerged side-note the only real commit sits ABOVE the merge-base and was never merged. content_merged_into_main cannot cheaply distinguish these, so the fix belongs in sync, not in that function.

FIX: in sync::detect, the merge-close passes (Case 1 'branch merged' AND Case 3 'branch content merged') must skip tickets whose state is pre-implementation. A ticket that never reached 'implemented' has no completed work that could have merged, so any merge signal there is a git-topology artifact. Define pre-implementation as states that occur before any code is written: new, groomed, specd, question. Skip those in both Case 1 and Case 3 (do not push them as close candidates).

CRITICAL — NO BEHAVIOR CHANGE FOR ANY OTHER CASE: Case 2 (implemented ticket on main with branch gone) is unaffected. Case 4 (target_branch merge) is unaffected. Tickets in implemented (and any post-implementation state) must still be detected and closed exactly as today. The hint-generation pass is unaffected. The only change is that Case 1 and Case 3 no longer close tickets in new/groomed/specd/question. Existing tests for implemented-ticket detection must continue to pass unchanged.

DO NOT change content_merged_into_main, merged_into_main, or any git_util function. The fix is purely a state filter in sync::detect. Add an integration test: a side-note ticket (state new) on a branch whose fork point is in main via an epic --no-ff merge must NOT appear in close candidates and must NOT generate a hint; and a regression test confirming an implemented ticket on a content-merged branch is still closed.

### Acceptance criteria

- [x] `apm sync detect` returns no close candidates for a ticket in state `new`, `groomed`, `specd`, or `question` whose branch's fork point is reachable from `main` only via a merge commit's non-first parent and whose branch contains no non-ticket-file commits (the side-note-on-epic-branch topology).
- [x] `apm sync detect` generates no hints for such a pre-implementation ticket (the hint pass already filters to `state == "implemented"`, and `merged_set` is populated correctly so Case 4 and the hint pass skip the branch).
- [x] `apm sync detect` still includes an `implemented` ticket in close candidates when its branch was merged into `main` via `--no-ff` (Case 1 regression).
- [x] `apm sync detect` still includes an `implemented` ticket in close candidates when its implementation commit was squash-merged into `main` with trailing state-only commits on the branch (Case 3 regression).
- [x] `git_util::content_merged_into_main`, `git_util::merged_into_main`, and all other `git_util` functions are unchanged.
- [x] All pre-existing `sync` integration tests pass without modification.
- [x] Two new integration tests are added to `apm/tests/integration.rs`: one reproducing the bug (pre-impl ticket not closed) and one confirming the regression (implemented ticket still closed).

### Out of scope

- Fixing `content_merged_into_main` to distinguish false positives from true merges at the git-util level.
- Protecting tickets in states other than `new`, `groomed`, `specd`, `question` (e.g. `in_design`, `ammend`, `ready`, `in_progress`, `blocked`) — those states may have real implementation commits that were legitimately merged.
- Changes to Case 2 (`implemented, branch gone`) or Case 4 (`branch merged into target_branch`) of `sync::detect`.
- Changes to the hint-generation pass (it already restricts to `state == "implemented"` and is unaffected).
- Changes to `apm-server` or `apm-ui`.

### Approach

The fix is two guard clauses in `apm-core/src/sync.rs` and two new integration tests. No other files change.

#### apm-core/src/sync.rs

At the top of `detect`, after `let terminal = config.terminal_state_ids();`, add:

```rust
const PRE_IMPL: &[&str] = &["new", "groomed", "specd", "question"];
```

**Case 1** (currently lines 58–59): insert a guard between the terminal check and the `close.push`:

```rust
if terminal.contains(t.frontmatter.state.as_str()) { continue; }
if PRE_IMPL.contains(&t.frontmatter.state.as_str()) { continue; }
close.push(CloseCandidate { ticket: t, reason: "branch merged" });
```

**Case 3** (currently lines 83–86): `merged_set.insert` must still run unconditionally (so later passes skip the branch), but `close.push` is guarded by both the existing terminal check and the new pre-impl check:

```rust
merged_set.insert(branch.clone());
let state = t.frontmatter.state.as_str();
if !terminal.contains(state) && !PRE_IMPL.contains(&state) {
    close.push(CloseCandidate { ticket: t, reason: "branch content merged" });
}
```

The hint-generation pass (checking `state == "implemented"`) is already safe and needs no change.

#### apm/tests/integration.rs — two new tests

**Test 1: `sync_detect_skips_pre_impl_ticket_with_fork_in_main`** (bug reproduction)

Build the topology that triggers the false positive:

1. `init_repo()` — main at commit A.
2. Checkout `epic/test` from A; add an allow-empty commit E1; return to main.
3. Checkout `epic/test` again; branch off to `ticket/aa11bb22-side-note`; write `tickets/aa11bb22-side-note.md` with `state = "new"` and no implementation content; commit T1; return to main.
4. `git merge --no-ff epic/test` — main is now M with parents A (first) and E1 (second).

Topology result: `merge_base(main, ticket_branch) = E1`, which is reachable from main only via M's second parent (not the first-parent chain). `content_merged_into_main` sees `content_tip = None` (all commits touch only `tickets/`) and the regular-merge sub-case incorrectly returns `true` — exactly the bug.

Assertions:
- `candidates.close` does not contain the ticket branch.
- `candidates.hints` is empty.

**Test 2: `sync_detect_implemented_ticket_still_closed_after_pre_impl_filter`** (Case 1 regression)

1. `write_ticket_to_branch(p, "implemented", "regression-test")` — uses the existing helper to create an implemented ticket.
2. Retrieve the branch; `git merge --no-ff <branch>` into main.
3. Call `sync::detect`.

Assertion: the ticket branch appears in `candidates.close` (reason `"branch merged"`), confirming the new PRE_IMPL filter does not block implemented tickets.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T00:18Z | — | new | philippepascal |
| 2026-05-29T00:20Z | new | groomed | philippepascal |
| 2026-05-29T00:20Z | groomed | in_design | philippepascal |
| 2026-05-29T00:27Z | in_design | specd | claude |
| 2026-05-29T00:35Z | specd | ready | philippepascal |
| 2026-05-29T00:35Z | ready | in_progress | philippepascal |
| 2026-05-29T00:41Z | in_progress | implemented | claude |
