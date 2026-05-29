+++
id = "ada017c0"
title = "sync close-eligibility must not depend on completion strategy (fix 7dab64ea regression)"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ada017c0-sync-close-eligibility-must-not-depend-o"
created_at = "2026-05-29T02:52:06.091678Z"
updated_at = "2026-05-29T02:52:06.091678Z"
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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T02:52Z | — | new | philippepascal |
