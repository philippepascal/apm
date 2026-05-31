+++
id = "c3f5aa4d"
title = "apm validate: enforce new workflow rules including trigger uniqueness"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c3f5aa4d-apm-validate-enforce-new-workflow-rules-"
created_at = "2026-05-31T01:58:46.595888Z"
updated_at = "2026-05-31T03:03:50.228533Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["1e758cd5"]
+++

## Spec

### Problem

Add validate-time enforcement for the workflow schema landed by 1e758cd5.

NEW RULES TO ENFORCE:

1. TRIGGER-UNIQUENESS (the load-bearing new rule). Any state that is the destination of a transition with a non-manual trigger (currently only 'command:start') must have exactly one incoming transition in the entire workflow. No other transition — triggered or manual — may land on a state that is reached via a triggered transition.

   Rationale: triggers mark a state as freshly ready for an external dispatcher (apm start, apm work, UI dispatcher) to pick up. If another transition can also land on that state, being in the state no longer reliably implies a fresh dispatch is needed. Enforcing uniqueness keeps the flag unambiguous.

   Error message should name both transitions that violate the rule and identify the destination state.

2. worker_profile shape. If a state declares worker_profile, the value must parse as agent/role (single slash, both halves non-empty). The role component must not be the literal string 'worker'.

3. command:start lands on dispatch-capable state. Every transition with trigger = 'command:start' must land on a state that has worker_profile set. A command:start transition pointing to a supervisor-owned state is meaningless (nothing to dispatch).

CONSOLIDATE EXISTING RULES (review what already exists in apm validate; keep behaviour, just ensure they still fire):
- Terminal states have no outgoing transitions.
- Every non-terminal state is reachable from the new state.
- Workflow has exactly one initial state ('new').

TESTS:
- A workflow where two transitions land on a state and one is command:start fails validate; error names both source states and the destination.
- A workflow where two transitions both have command:start and land on the same state fails validate (same rule, different angle).
- A workflow with state.worker_profile = 'claude/worker' fails validate ('worker' is reserved).
- A workflow with state.worker_profile = 'claudecoder' (no slash) fails validate.
- A workflow with state.worker_profile = '/coder' (empty agent) fails validate.
- A workflow with command:start to a state without worker_profile fails validate.
- The default workflow (after 1e758cd5 lands) passes validate.
- This project's .apm/workflow.toml (after 1e758cd5 lands) passes validate.

OUT OF SCOPE:
- Schema struct changes (covered by 1e758cd5).
- Dispatch path consumer updates (separate ticket).
- Instructions filter (separate ticket).
- External-project migration tooling (separate ticket).

REFERENCES:
- apm-core/src/validate.rs (or wherever apm validate lives — check apm-core source tree)
- apm-core/src/config.rs for the new struct shape (after 1e758cd5)
- This is depends_on 1e758cd5 because the validate code needs to operate on the new struct fields.

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
| 2026-05-31T01:58Z | — | new | philippepascal |
| 2026-05-31T03:03Z | new | closed | philippepascal |
