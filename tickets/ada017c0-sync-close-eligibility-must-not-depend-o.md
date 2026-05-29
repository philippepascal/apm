+++
id = "ada017c0"
title = "sync close-eligibility must not depend on completion strategy (fix 7dab64ea regression)"
state = "ready"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ada017c0-sync-close-eligibility-must-not-depend-o"
created_at = "2026-05-29T02:52:06.091678Z"
updated_at = "2026-05-29T03:30:04.397402Z"
+++

## Spec

### Problem

REGRESSION (main is currently RED): ticket 7dab64ea changed sync::detect's close-eligibility gate to an allowlist derived from the completion strategy — Config::merge_completed_state_ids() returns the 'to' targets of transitions whose completion is Pr/Merge/PrOrEpicMerge. This broke the external-PR-merge-then-sync flow. The pre-existing e2e test apm/tests/e2e.rs::full_ticket_lifecycle fails at e2e.rs:568 ('merge suggestion not reported'): cargo test --workspace fails on main.

ROOT CAUSE: sync::detect answers two INDEPENDENT questions per ticket. (1) 'Is the branch merged into main?' — pure git topology (merged_into_main / content_merged_into_main); identical for a PR-merge and a direct merge; CORRECT and unchanged. (2) 'Is the ticket in a state where being-merged means close it?' — the eligibility gate. 7dab64ea keyed question 2 on the completion strategy, but the completion strategy describes HOW apm performs the merge at the in_progress->implemented transition, NOT whether the state is a done/closeable state. A workflow whose implemented transition uses completion = "none" (the merge is done externally — a human merges the PR on GitHub) therefore has an EMPTY merge_completed set, so sync closes nothing. The default APM workflow uses pr_or_epic_merge so default users do not see the regression; the e2e test models completion = "none" (external merge) and exposes it.

SUPERVISOR DECISION (already made): the external-PR-merge-then-sync flow is legitimate. 'implemented + merged-by-PR' and 'implemented + merged-directly' are the SAME git fact and must be detected by the SAME mechanism. Close-eligibility must NOT depend on whether apm itself performed the merge.

GOAL: decouple the close-eligibility gate from the completion strategy. Re-derive the set of states eligible for close-on-merge (equivalently: the set of pre-implementation states to EXCLUDE) from a signal that holds regardless of how the merge happens. This restores the behavior that 39b9c568's denylist provided (close any merged non-terminal ticket except pre-implementation states like new/groomed/specd/question) while keeping 7dab64ea's anti-hardcoding goal (no string-literal state IDs in sync.rs).

HARD CONSTRAINTS:
- Do NOT key eligibility on CompletionStrategy. merge_completed_state_ids() (added by 7dab64ea, used only in sync.rs) should be replaced/removed.
- Do NOT hardcode state-ID string literals in sync.rs (preserve 7dab64ea's goal).
- KEEP the side-note false-positive fix (ticket 39b9c568): pre-implementation tickets (new/groomed/specd/question equivalents) whose fork-point reached main via an unrelated merge must NOT be closed.
- KEEP the terminal-state exclusion across all passes (the 7dab64ea amendment): never produce a close candidate or hint for a terminal ticket.
- The e2e test full_ticket_lifecycle (completion = "none", external --no-ff merge of an implemented ticket) MUST pass and MUST NOT be weakened to mask the bug — it is the regression witness.
- Detection functions (merged_into_main, content_merged_into_main, is_branch_merged_into) are correct; do not change them.

RECOMMENDED DIRECTION (spec-writer to evaluate and refine — not prescriptive): derive eligibility from workflow STRUCTURE/POSITION or ticket HISTORY rather than completion strategy. Two candidates to weigh: (a) identify the implementation entry state as the 'to' of the command:start transition whose role is the coder/implementer, and exclude non-terminal states that strictly precede it in the workflow graph (pre-implementation); (b) treat a ticket as eligible iff its History shows it has entered the implementation state at least once (it actually had a worker write code), which is naturally false for side-notes that never left 'new'. Pick whichever is cleaner; validate against BOTH the default workflow (pr_or_epic_merge -> implemented) AND the completion="none" e2e workflow. Re-examine the tests 7dab64ea added (the custom 'shipped' workflow test and the terminal-overlap test) and adapt them to the new model.

OUT OF SCOPE: fixing the deeper content_merged_into_main false positive (a branch with no implementation commits whose merge_base reached main via a side-parent) — that is the hard git-analysis problem we deliberately deferred; the state/position guard remains the pragmatic approach. No changes to apm-server or apm-ui.

### Acceptance criteria

- [ ] `cargo test --workspace` passes after the fix; the `full_ticket_lifecycle` e2e test passes without any modification to the test itself
- [ ] `apm sync` on a workflow where `in_progress → implemented` uses `completion = "none"` reports "branch merged" for an `implemented`-state ticket whose branch was `--no-ff` merged into main (the regression witness; covered by the e2e test, whose ticket has `in_progress` in its history)
- [ ] `apm sync` produces no close candidate for a `new`-state ticket whose branch fork reaches main via an epic `--no-ff` merge (side-note guard preserved)
- [ ] `apm sync` produces no close candidate for a `ready`-state ticket that never entered implementation, even when its branch is merged into main
- [ ] `apm sync` produces no close candidate and emits no hint for a terminal-state ticket even when its branch is merged
- [ ] `Config::implementation_state_ids()` returns the union of (a) `to` targets of `command:start` transitions whose `worker_profile` does not end with `/spec-writer`, and (b) `to` targets of `Pr`/`Merge`/`PrOrEpicMerge` completion transitions; for the default workflow it returns `{in_progress, implemented}`
- [ ] `Config::implementation_state_ids()` is invariant to the order in which `[[workflow.states]]` are listed — a unit test builds the default workflow, then builds it again with the state entries shuffled, and asserts an identical result
- [ ] A `completion = "none"` workflow still yields a non-empty `implementation_state_ids()` (`{in_progress}` via the coder `command:start` signal), and a workflow with no `command:start` but a merge-completion transition yields the merge target — neither signal alone can empty the set
- [ ] `ticket_fmt::history_target_states(body)` returns the `To` column of every History row (header and separator rows skipped); unit-tested
- [ ] `sync::detect` eligibility is computed per-ticket as `implementation_state_ids().contains(current_state) || any(history "To" in implementation_state_ids())`; an integration test asserts `detect`'s close candidates are identical when `[[workflow.states]]` are shuffled
- [ ] `Config::merge_completed_state_ids()` is removed; `sync.rs` contains no string-literal state IDs for close-eligibility and no dependence on state list order

### Out of scope

- The deeper `content_merged_into_main` false positive (a branch with no implementation commits whose merge_base reaches main via an epic's non-first parent) — deferred as before
- Changes to detection functions `merged_into_main`, `content_merged_into_main`, `is_branch_merged_into` — correct and untouched
- Changes to `apm-server` or `apm-ui`
- History-based eligibility (approach b from the problem statement) — discarded in favour of workflow-structure approach
- User-visible CLI or UI changes to expose the pre-implementation state concept

### Approach

The eligibility gate is decoupled from BOTH the completion strategy (the 7dab64ea bug) AND config order. A ticket is eligible for close-on-merge iff it has actually reached an implementation state — answered from the ticket's current state and its own History, never from where a state is listed in config.

#### 1. `apm-core/src/config.rs` — add `implementation_state_ids()`, remove `merge_completed_state_ids()`

A state counts as an "implementation state" if a ticket can only be in it once real implementation work exists. Two order-independent signals identify these, and their UNION is used so no single signal can empty the set:

- the `to` target of any `command:start` transition whose `worker_profile` does NOT end with `/spec-writer` (a coder/implementer entry; `None` profile counts as non-spec-writer);
- the `to` target of any transition whose `completion` is `Pr`, `Merge`, or `PrOrEpicMerge` (a merge-completion target).

```rust
pub fn implementation_state_ids(&self) -> std::collections::HashSet<String> {
    self.workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .filter(|t| {
            let is_coder_start = t.trigger == "command:start"
                && t.worker_profile.as_deref().map_or(true, |p| !p.ends_with("/spec-writer"));
            let is_merge_completion = matches!(t.completion,
                CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge);
            is_coder_start || is_merge_completion
        })
        .map(|t| t.to.clone())
        .collect()
}
```

This is a SET built from transition fields; it does not read the position of any `[[workflow.states]]` entry. Remove `merge_completed_state_ids()` entirely.

Why this is NOT the 7dab64ea regression: completion strategy is only one of two unioned signals. In the `completion = "none"` workflow the merge-completion signal contributes nothing, but the coder `command:start` signal still yields `{in_progress}`, so the set is non-empty and detection keeps working (via History, below). Completion strategy can no longer be the sole gate that empties the set.

Resulting sets:
- default workflow: `{in_progress, implemented}` (coder start → in_progress; pr_or_epic_merge → implemented)
- the e2e `completion = "none"` workflow: `{in_progress}` (coder start only; no merge completion)
- a custom workflow with no `command:start` but `in_progress → shipped` (`completion = "merge"`): `{shipped}`

Unit tests: default → `{in_progress, implemented}`; order-invariance (build the default workflow, then build it again with the `[[workflow.states]]` entries shuffled, assert identical result); a `command:start` transition with no `worker_profile` is treated as a coder entry.

#### 2. `apm-core/src/ticket/ticket_fmt.rs` — History "To"-column parser

`Ticket` already exposes `pub body: String`. Add:

```rust
pub fn history_target_states(body: &str) -> Vec<String> {
    let Some(idx) = body.find("\n## History") else { return Vec::new() };
    body[idx..].lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('|') { return None; }
            // cols = ["", When, From, To, By, ""]; the To column is index 3
            let to = line.split('|').map(str::trim).nth(3)?.to_string();
            if to.is_empty() || to == "To" || to.chars().all(|c| c == '-') { return None; }
            Some(to)
        })
        .collect()
}
```

Unit test: a body with header, separator, and two data rows returns just the To column (e.g. `["new", "in_progress", "implemented"]`), header/separator skipped.

#### 3. `apm-core/src/sync.rs` — eligibility = reached an implementation state

Replace line 28 (`let merge_completed = config.merge_completed_state_ids();`) with:

```rust
let impl_states = config.implementation_state_ids();
let eligible = |t: &Ticket| -> bool {
    impl_states.contains(t.frontmatter.state.as_str())
        || crate::ticket_fmt::history_target_states(&t.body)
            .iter().any(|s| impl_states.contains(s.as_str()))
};
```

A ticket is eligible iff its current state is an implementation state OR its History shows it entered one at least once. This asks "did this ticket ever reach implementation" — immune to back-edges and cancel-edges (no graph reachability) and to state ordering. The current-state arm also covers tickets force-set to a done state without an intervening `in_progress` history row.

At the five gates replace `merge_completed.contains(state)` with `eligible(&t)`, keeping the terminal check (`state` stays `t.frontmatter.state.as_str()`):
- Case 1 (line 60): `if terminal.contains(state) || !eligible(&t) { continue; }`
- Case 3 (line 87): `if !terminal.contains(state) && eligible(&t) {`
- Case 2 (line 106): `if eligible(&t) && !terminal.contains(state) {`
- Case 4 (line 128): `if !eligible(&t) || terminal.contains(state) { continue; }`
- Hints (line 153): `if eligible(&t) && !terminal.contains(state) {`

No string-literal state IDs remain in sync.rs.

#### 4. Tests — `apm/tests/integration.rs`

- `sync_detect_skips_pre_impl_ticket_with_fork_in_main` (side-note `new`): passes unchanged — a `new` ticket's history only contains `new`, never an implementation state.
- `sync_detect_implemented_ticket_still_closed_after_pre_impl_filter`: passes — `implemented` is in the default `impl_states`, so the current-state arm makes it eligible even if the helper force-jumps without an `in_progress` row.
- `sync_detect_skips_non_merge_completed_ticket_on_merged_branch` (ready): update the pre-condition assertion to `!config.implementation_state_ids().contains("ready")`. A `ready` ticket is not an implementation state and (force-created) has no `in_progress` history → not eligible. Assertions hold.
- `sync_detect_uses_config_derived_merge_completed_state` (custom `shipped` workflow): replace the `merge_completed_state_ids()` assertion with `assert_eq!(config.implementation_state_ids(), {"shipped"})`. Ticket A (`state = "shipped"`) is eligible via the current-state arm (`shipped` ∈ `impl_states`); Ticket B (`state = "ready"`) is not. No fixture/history rewrite needed.
- `sync_detect_no_candidate_for_terminal_merge_completed_ticket`: keep — `done` ∈ `impl_states` but is terminal, so the terminal guard excludes it. Update the pre-condition to assert `config.terminal_state_ids().contains("done")`.
- ADD `sync_eligibility_invariant_to_state_order`: build a workflow plus a merged `implemented` ticket, run `detect`, then rebuild with `[[workflow.states]]` shuffled and run `detect` again; assert identical close candidates. Guards against reintroducing order dependence.

### Open questions


### Amendment requests

- [x] Order-independence requirement — INCORPORATED. The Approach no longer uses config list-position. `implementation_state_ids()` is built from transition fields (`trigger`, `worker_profile`, `completion`) as a set, and eligibility is decided from the ticket's current state plus its History "To" column. Acceptance criteria include a config-order shuffle-invariance unit test and a detect-level shuffle-invariance integration test.
- [x] Correction note (dropped words) — INCORPORATED; superseded by the rewritten Approach, which states the implementation-state signals and the History parser explicitly.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T02:52Z | — | new | philippepascal |
| 2026-05-29T02:53Z | new | groomed | philippepascal |
| 2026-05-29T02:55Z | groomed | in_design | philippepascal |
| 2026-05-29T03:06Z | in_design | specd | claude |
| 2026-05-29T03:14Z | specd | ammend | philippepascal |
| 2026-05-29T03:14Z | ammend | in_design | philippepascal |
| 2026-05-29T03:19Z | in_design | specd | claude |
| 2026-05-29T03:30Z | specd | ready | philippepascal |
