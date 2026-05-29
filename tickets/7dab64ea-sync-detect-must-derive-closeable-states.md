+++
id = "7dab64ea"
title = "sync::detect must derive closeable states from config, not hardcoded IDs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7dab64ea-sync-detect-must-derive-closeable-states"
created_at = "2026-05-29T00:56:29.083955Z"
updated_at = "2026-05-29T00:56:29.083955Z"
+++

## Spec

### Problem

PROBLEM: apm-core/src/sync.rs hardcodes workflow state IDs in six places, all of which are config-defined values from workflow.toml [[workflow.states]] and are renamable/replaceable per project:
- line 28: const PRE_IMPL = [new, groomed, specd, question] (added by ticket 39b9c568)
- line 60: PRE_IMPL guard in Case 1 (branch merged)
- line 87: PRE_IMPL guard in Case 3 (branch content merged)
- line 105: Case 2 checks state == "implemented"
- line 126: Case 4 checks state != "implemented"
- line 150: hint pass checks state == "implemented"

A project that renames these states (or uses a different state vocabulary) silently loses correct behavior with no error. The codebase already shows the right pattern at line 27: let terminal = config.terminal_state_ids(); — terminal-ness is read from a config flag (terminal = true in workflow.toml), not a literal. Every state-based decision in sync.rs should be config-derived the same way.

GOAL: replace ALL hardcoded state-ID logic in sync.rs with config-derived determinations. No string-literal state IDs should remain in sync.rs.

MECHANISM (for the spec-writer to refine): the branch-merge close passes (Case 1, Case 3) and the implemented-specific passes (Case 2, Case 4, hint pass) all hinge on the same underlying notion: a ticket is eligible for merge-based auto-close only once it has reached the state produced by a merging completion. That state is config-derivable: it is the 'to' target of any transition whose completion strategy is a merging one (Merge / Pr / PrOrEpicMerge). In the default workflow that transition is in_progress -> implemented (completion = pr_or_epic_merge), so the eligible state is 'implemented'. Add a Config helper analogous to terminal_state_ids() (e.g. one that returns the set of states reached via a merging completion strategy) and use it everywhere sync currently hardcodes a state name. Case 2 and Case 4's 'implemented' checks and the hint pass's 'implemented' check should consult the same config-derived set rather than the literal.

SUPERVISOR DECISION (already made — do not re-litigate): sync should auto-close ONLY tickets that have reached a merge-completion state. This is STRICTER than the behavior ticket 39b9c568 left in place (which still closed any non-terminal merged ticket in states like in_design/ammend/ready/in_progress/blocked). That broadening is intended and correct: a ticket that never reached a merge-completion state has no APM-merged work, so any merge signal is a git-topology artifact. Cases 1 and 3 should therefore only produce close candidates for tickets in the config-derived merge-completed set (plus whatever the spec-writer determines for the genuinely-merged terminal/edge cases).

OUT OF SCOPE: changing the git merge-detection functions (merged_into_main, content_merged_into_main, is_branch_merged_into) — those are correct and unchanged. This is a pure config-derivation refactor of sync::detect plus a new Config helper.

TESTS: existing sync integration tests must still pass. New/updated tests must not hardcode state IDs in a way that bakes in the default workflow's names where avoidable; assert behavior via the config-derived helper. Confirm the side-note false-positive (ticket 39b9c568's scenario) stays fixed and that an implemented merged ticket is still closed.

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
| 2026-05-29T00:56Z | — | new | philippepascal |
