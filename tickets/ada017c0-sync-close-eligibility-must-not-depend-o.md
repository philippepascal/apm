+++
id = "ada017c0"
title = "sync close-eligibility must not depend on completion strategy (fix 7dab64ea regression)"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ada017c0-sync-close-eligibility-must-not-depend-o"
created_at = "2026-05-29T02:52:06.091678Z"
updated_at = "2026-05-29T03:14:18.771143Z"
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

- [ ] `cargo test --workspace` passes after the fix; the `full_ticket_lifecycle` e2e test passes without modification to the test itself
- [ ] `apm sync` on a workflow where `in_progress → implemented` uses `completion = "none"` reports "branch merged" for an `implemented`-state ticket whose branch was `--no-ff` merged into main
- [ ] `apm sync` produces no close candidate for a `new`-state ticket whose branch fork reaches main via an epic `--no-ff` merge (the pre-implementation false-positive guard still applies)
- [ ] `apm sync` produces no close candidate for a `ready`-state ticket (default workflow) whose branch is merged directly into main
- [ ] `apm sync` produces no close candidate and emits no hint for a terminal-state ticket even when its branch is merged
- [ ] `Config::pre_implementation_state_ids()` returns all states preceding the coder entry state in config order; for the default workflow this is `{new, groomed, question, specd, ammend, in_design, ready}`
- [ ] `Config::pre_implementation_state_ids()` falls back to the first non-None completion source when no coder `command:start` exists; for the custom `shipped` workflow (`in_progress → shipped` with `completion = "merge"`) it returns `{"ready"}`
- [ ] `Config::merge_completed_state_ids()` is removed from `config.rs`; `sync.rs` retains no hardcoded state-ID string literals for close-eligibility

### Out of scope

- The deeper `content_merged_into_main` false positive (a branch with no implementation commits whose merge_base reaches main via an epic's non-first parent) — deferred as before
- Changes to detection functions `merged_into_main`, `content_merged_into_main`, `is_branch_merged_into` — correct and untouched
- Changes to `apm-server` or `apm-ui`
- History-based eligibility (approach b from the problem statement) — discarded in favour of workflow-structure approach
- User-visible CLI or UI changes to expose the pre-implementation state concept

### Approach

The fix has three parts: a new Config helper, a sync.rs swap, and updates to three integration tests.

#### 1. `apm-core/src/config.rs` — replace `merge_completed_state_ids()` with `pre_implementation_state_ids()`

Add `pub fn pre_implementation_state_ids(&self) -> std::collections::HashSet<String>` to `impl Config`, directly after `terminal_state_ids()`. Remove `merge_completed_state_ids()` entirely.

Algorithm (tried in order):

**Method 1 — coder `command:start` entry:** Walk `self.workflow.states` in config order. For each state, find the first transition where `trigger == "command:start"` and `worker_profile` does NOT end with `"/spec-writer"` (treat `None` as non-spec-writer). Look up the `to` state's index in `self.workflow.states`. Pre-implementation = all states at indices strictly less than that index.

**Method 2 — first non-None completion source (fallback):** Walk states in config order. Find the first state that has any transition with `completion != CompletionStrategy::None`. That state's config index is the dividing line. Pre-implementation = all states at indices strictly less than that index.

**Method 3 — no signal:** Return `HashSet::new()` (all non-terminal states are eligible).

The returned set contains only state IDs (strings), never the implementation-entry state itself.

Update the unit test block: remove `merge_completed_state_ids_returns_correct_set`. Add:
- `pre_implementation_state_ids_default_workflow`: build a config with the default workflow shape (groomed/ammend → in_design via spec-writer command:start; ready → in_progress via coder command:start); assert the result is `{new, groomed, question, specd, ammend, in_design, ready}`.
- `pre_implementation_state_ids_shipped_workflow`: build the custom shipped workflow (no command:start; in_progress → shipped with `completion = "merge"`); assert the result is `{"ready"}`.
- `pre_implementation_state_ids_no_signal`: build a workflow with only `manual` transitions and `completion = "none"`; assert the result is empty.

#### 2. `apm-core/src/sync.rs` — swap `merge_completed` for `pre_impl`

Change line 28:
```
let merge_completed = config.merge_completed_state_ids();
```
to:
```
let pre_impl = config.pre_implementation_state_ids();
```

At each of the five eligibility gates, replace `merge_completed.contains(state)` with `!pre_impl.contains(state)` (and flip negations consistently):

- Case 1 (line 60): `|| !merge_completed.contains(state)` → `|| pre_impl.contains(state)`
- Case 3 (line 87): `&& merge_completed.contains(state)` → `&& !pre_impl.contains(state)`
- Case 2 (line 106): `merge_completed.contains(state) &&` → `!pre_impl.contains(state) &&`
- Case 4 (line 128): `!merge_completed.contains(state) ||` → `pre_impl.contains(state) ||`
- Hints (line 153): `merge_completed.contains(state) &&` → `!pre_impl.contains(state) &&`

#### 3. `apm/tests/integration.rs` — adapt the three tests that call `merge_completed_state_ids()`

`sync_detect_skips_non_merge_completed_ticket_on_merged_branch` (line 7673): replace the pre-condition assertion — `!config.merge_completed_state_ids().contains("ready")` → `config.pre_implementation_state_ids().contains("ready")`. No other changes; the ticket A/B assertions still hold.

`sync_detect_uses_config_derived_merge_completed_state` (line 7733): replace the `assert_eq!(config.merge_completed_state_ids(), ...)` block with `assert_eq!(config.pre_implementation_state_ids(), ["ready".to_string()].into_iter().collect::<std::collections::HashSet<_>>(), ...)`. The ticket A/B candidate assertions remain unchanged — both still hold under the new model because Method 2 identifies `ready` as pre-implementation and `shipped` as eligible.

`sync_detect_no_candidate_for_terminal_merge_completed_ticket` (line 7816): replace `assert!(config.merge_completed_state_ids().contains("done"), ...)` with `assert!(config.terminal_state_ids().contains("done"), ...)`. The candidate assertion remains: `done` is terminal, so it is excluded by the existing terminal guard regardless of the new pre-impl logic.

No changes to the fourth test `sync_detect_skips_pre_impl_ticket_with_fork_in_main` or `sync_detect_implemented_ticket_still_closed_after_pre_impl_filter` — they exercise the correct behaviour and pass with the new logic without modification.

### Open questions


### Amendment requests

- [ ] HARD CONSTRAINT: the close-eligibility classification MUST be invariant to the order in which [[workflow.states]] are listed in config. The current Approach is REJECTED: both Method 1 and Method 2 use config list-position ('states at indices strictly less than the entry index') as a proxy for lifecycle progression. A project that lists its states out of lifecycle order would get a silently-wrong result. Re-derive eligibility from a signal that does not depend on state list order.

REQUIRED INVARIANCE TEST: add a test that builds a workflow, computes the pre-implementation / eligible classification, then rebuilds the SAME workflow with the [[workflow.states]] entries shuffled into a different order, recomputes, and asserts the classification is identical. This test must exist and pass — it is the guard against reintroducing order dependence. Replace AC6/AC7 (which assert specific sets derived from config order) with order-independent assertions plus this invariance test.

IMPL-ENTRY IDENTIFICATION must also be order-independent: identify the implementation-entry state(s) as the SET of  targets of transitions whose trigger == "command:start" and whose worker_profile does NOT end with "/spec-writer" (set membership — NOT 'the first one in config order'). For the default workflow this set is {in_progress}.

RECOMMENDED APPROACH (history-based — cleanest order- AND cycle-independent option): a ticket is eligible for close-on-merge iff its ## History shows it has entered an implementation-entry state at least once (i.e. some History row's 'To' column is in the impl-entry set). Rationale: current-state classification cannot distinguish 'ready, never worked' (a side-note-like ticket, must NOT close) from 'ready, reverted after being worked' — only history disambiguates, and history is exactly 'did this ticket ever reach implementation'. This is immune to back-edges (in_progress->ready) and cancel-edges (X->closed) that make pure graph reachability unusable, and it is independent of state list order. NOTE: there is no existing history-row parser — the body is only split at '## History' (ticket_fmt.rs:187). A small parser is needed (read rows after '## History', take the 'To' column). The spec-writer should add a minimal helper (ideally on Ticket or in ticket_fmt) and unit-test it.

IF a state-based or graph-based approach is chosen instead of history: it MUST still pass the shuffle-invariance test AND correctly handle cycles (back-edges and cancel/closed shortcut edges) — naive 'reachable from / can reach impl-entry' does NOT work because the default workflow has in_progress->ready back-edges and new->closed cancel edges. Do not rely on the  field being present unless you also handle its absence.

ALL PRIOR CONSTRAINTS STILL HOLD: keep the side-note false-positive guard (a  ticket whose fork reached main via an epic merge must not close); keep terminal-state exclusion in all passes; the e2e full_ticket_lifecycle test (completion="none", external merge of an implemented ticket) MUST pass unmodified; no hardcoded state-ID string literals in sync.rs; remove merge_completed_state_ids(); validate against the default workflow, the custom 'shipped' workflow, AND the completion="none" workflow.

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