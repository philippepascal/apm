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

When a ticket lands in a merge-failure state (e.g. `merge_failed` in the default workflow, though the state name is project-configurable), the supervisor has no in-context guidance on how to proceed. `apm show` prints frontmatter and history without surfacing recovery options. `apm list` filtered to the failure state prints rows with no hint. `apm next` can surface a merge-failure ticket as actionable without explaining what action to take. The supervisor must either know the conventions from memory or consult external documentation.

With config-aware surfacing, the CLI derives recovery options directly from the workflow configuration: which transition retries the merge, which returns the ticket to a worker, and which abandons it. All labels and target state IDs come from config, enforcing the order-independence discipline established by tickets ada017c0 and 27439a80 — no state name is hardcoded anywhere in the output path.

### Acceptance criteria

- [ ] `classify_recovery_options(state_id, config)` classifies a transition as `RetryMerge` when its to-state is the target of at least one merging-completion transition (Pr, Merge, or PrOrEpicMerge) anywhere in the workflow
- [ ] `classify_recovery_options` classifies a transition as `ReturnToWorker` when its to-state is the target of at least one non-spec-writer `command:start` transition anywhere in the workflow
- [ ] `classify_recovery_options` classifies a transition as `Abandon` when its to-state has `terminal: true`
- [ ] `classify_recovery_options` classifies a transition as `Other` when none of the above apply
- [ ] Each `RecoveryOption` carries: to-state ID, display label (from `transition.label`, falling back to to-state ID when label is empty), and `RecoveryKind`
- [ ] Results are ordered by `workflow.states` declaration order; classification is independent of that order (shuffling the states list produces identical results)
- [ ] Against the default workflow, `classify_recovery_options("merge_failed", config)` returns `implemented` as `RetryMerge` and `in_progress` as `ReturnToWorker`
- [ ] Against a workflow where the merge-target state is renamed (e.g. `implemented` → `shipped`), the helper classifies `shipped` as `RetryMerge`
- [ ] When the queried state has no transitions to merge-target states, `classify_recovery_options` returns no `RetryMerge` entries
- [ ] `apm show <id>` prints a "Recovery options" block when the ticket's current state has any `RetryMerge` transitions OR the ticket body contains a section headed "Merge notes"
- [ ] The recovery block in `apm show` lists each option with its display label and the exact command `apm state <id> <to>`, and includes a reference to `docs/merge-failed-recovery.md`
- [ ] `apm show <id>` does not print a recovery block when no `RetryMerge` transitions exist and the body contains no "Merge notes" section
- [ ] `apm list --state <STATE>` appends a one-line recovery summary below ticket rows when STATE has `RetryMerge` transitions; omits the summary otherwise
- [ ] `apm next` (plain-text mode) prints recovery options below the ticket line when the selected ticket's state has `RetryMerge` transitions; JSON mode output is unchanged

### Out of scope

- Terminal hint printed immediately after `apm state` completes (deliberately dropped — the high-value surfaces are the ones the supervisor reaches during triage)
- `apm work` / dispatcher changes (the dispatcher already treats merge-failure states as supervisor-actionable)
- Auto-recovery, auto-retry, or action buttons of any kind
- `apm-server` / `apm-ui` surfaces (covered by a separate ticket)
- Adding or modifying `[[workflow.states]]` entries or transitions in `workflow.toml`
- Hardcoding any state name or state ID as a string literal in CLI output paths

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