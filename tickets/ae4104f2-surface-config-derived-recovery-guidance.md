+++
id = "ae4104f2"
title = "Surface config-derived recovery guidance for merge-failure states in apm CLI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ae4104f2-surface-config-derived-recovery-guidance"
created_at = "2026-05-30T02:11:03.737221Z"
updated_at = "2026-05-30T02:14:18.952503Z"
+++

## Spec

### Problem

GOAL: when a ticket is stuck in a state that came from a failed merge (e.g. merge_failed in the default workflow, but the state name is configured per project), the supervisor should see clear, workflow-correct recovery options in the apm CLI without having to remember 'apm state X implemented'. The state names are derived from each project's workflow.toml, so all guidance must be config-derived and order-independent (no string-literal state IDs — the discipline ada017c0 and 27439a80 established).

PROBLEM: today a merge_failed ticket gives the supervisor no in-context guidance. apm show prints frontmatter and history but does not surface the recovery options. apm list filtered to merge_failed prints rows with no hint. apm next can land on a merge_failed ticket as actionable without explaining what action to take. The supervisor either knows the conventions (apm state X implemented to retry, apm state X in_progress to return to a worker, apm state X closed to abandon) or has to read docs/merge-failed-recovery.md to find them. With config-aware surfacing the same flow becomes self-documenting.

APPROACH (direction; spec-writer to refine):
1. Add a helper to apm-core that takes a state ID and returns the list of outgoing transitions classified by intent. The classification reads transition fields (no hardcoding state names):
   - retry-merge: transition whose completion is Pr, Merge, or PrOrEpicMerge — this is the path that re-attempts the merge after the supervisor resolves
   - return-to-worker: transition whose to-state is in implementation_state_ids and whose completion is None — sends the ticket back to an implementing state without auto-merge
   - abandon: transition whose to-state is in terminal_state_ids (this is the universal close path; closed is always available via the built-in terminal state even if not listed in workflow states)
   - other: any remaining transitions surfaced without classification
   Return the to-state ID plus the configured label (transition.label, falling back to the to-state ID if unset) for each. Order of the returned list mirrors workflow.states order, but classification is order-independent (driven by transition fields).
2. Surface the helper in three CLI commands:
   - apm show ID: when the ticket's current state has any outgoing transitions with completion in the merging set OR the ticket has a Merge notes body section (the latter is the signal that a merge previously failed regardless of workflow shape), print a Recovery options block listing each option with its label and the exact apm CLI command the supervisor would run. Also point to docs/merge-failed-recovery.md.
   - apm list --state STATE: when STATE matches the on_failure target of a merging transition (or equivalently a state with outgoing retry-merge transitions), append a one-line summary at the end of the rows describing the recovery options derived once for that state.
   - apm next: when the chosen ticket's state has retry-merge transitions available, print the recovery options after the ticket line so the supervisor sees them without a separate apm show call.

OUT OF SCOPE:
- state.rs terminal hint after a transition fires (deliberately dropped — by the time the supervisor sees the result they already invoked the command; the high-value surfaces are the ones the supervisor reaches by intent during triage).
- apm work / dispatcher integration. The dispatcher already treats merge_failed states as supervisor-actionable; behavior changes belong in a separate ticket if needed.
- Adding action buttons or auto-recovery anywhere.
- Hardcoding any state name in CLI output (the helper must drive all labels and target states from config).
- apm-server / apm-ui (a separate ticket covers UI surfacing of the same helper output).

TESTS:
- Helper unit tests: against the default workflow, the helper on merge_failed returns implemented as retry-merge, in_progress as return-to-worker (its completion is None and it is in implementation_state_ids), and closed as abandon (terminal). Against a workflow that renames the merge-completion state to shipped, the same helper returns shipped as retry-merge — invariant to state list order (build the same workflow with the [[workflow.states]] entries shuffled and assert identical output).
- Helper unit tests: against a workflow with no merging completions on the current state, the helper returns no retry-merge entries and the CLI commands suppress the retry block.
- Integration tests for the three CLI surfaces: apm show on a merge_failed ticket prints the recovery block with the right apm state commands; apm list --state merge_failed prints the summary; apm next selecting a merge_failed ticket prints the hint.

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
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:14Z | groomed | in_design | philippepascal |
