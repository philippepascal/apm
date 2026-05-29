+++
id = "3efea02e"
title = "apm validate: reject merging-completion transition targeting a terminal state"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3efea02e-apm-validate-reject-merging-completion-t"
created_at = "2026-05-29T01:28:21.747382Z"
updated_at = "2026-05-29T01:28:21.747382Z"
+++

## Spec

### Problem

apm validate (validate_config_no_agents in apm-core/src/validate.rs) already validates transition-level rules in the loop around line 375: target state must exist, Merge/PrOrEpicMerge transitions require on_failure, terminal states must have no outgoing transitions, etc.

ADD A RULE: a transition whose completion strategy is Pr, Merge, or PrOrEpicMerge must NOT target a terminal state. This should be an ERROR (consistent with the existing 'terminal but has outgoing transitions' error), not a warning.

RATIONALE: a merging completion is meant to produce a state the supervisor reviews before closing (the implemented -> closed gate). If a merging transition targets a terminal state directly, it collapses merge-and-close, removing the review gate, and makes sync's merge-completed-state set overlap the terminal set — the exact degenerate config that ticket 7dab64ea's runtime guard defends against. Supervisor decision: merge-straight-to-terminal is treated as a misconfiguration and rejected at config time.

WHERE: add the check inside the existing per-transition loop in validate_config_no_agents, alongside the on_failure check (around line 405-417). The set of terminal state IDs is available via config.terminal_state_ids() (note 'closed' is a built-in terminal state even if absent from [[workflow.states]], so the check must treat transition.to == "closed" as terminal too, mirroring how the existing target-exists check special-cases "closed").

MESSAGE FORMAT: consistent with sibling errors, e.g. 'config: state.{state_id}.transition({to}) — completion {strategy} targets terminal state {to}; merging completions must target a non-terminal (review) state'.

RELATIONSHIP: complementary to ticket 7dab64ea (which makes sync robust at runtime by subtracting terminal from the merge-completed set in all passes). This ticket is the config-time safety net; the two are independent and touch different files (validate.rs vs sync.rs/config.rs).

OUT OF SCOPE: changes to sync.rs (covered by 7dab64ea); apm validate --fix auto-remediation for this rule (no single safe fix — resolving it means either retargeting the transition or making the state non-terminal, both supervisor decisions); validating observation 2 (a merge-completed state that is also a normal mid-flow state) — deliberately not validated, too fuzzy to define crisply.

TESTS: validate_config rejects a workflow with a Merge or PrOrEpicMerge transition targeting a terminal state (including the built-in 'closed'); validate_config accepts the default workflow (in_progress -> implemented, where implemented is non-terminal). Existing validate tests must still pass.

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
| 2026-05-29T01:28Z | — | new | philippepascal |
