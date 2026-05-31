+++
id = "52d2a814"
title = "apm validate: enforce trigger-uniqueness and worker_profile shape"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/52d2a814-apm-validate-enforce-trigger-uniqueness-"
created_at = "2026-05-31T02:57:37.160432Z"
updated_at = "2026-05-31T02:57:37.160432Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["071886fc"]
+++

## Spec

### Problem

STEP 5 of the incremental workflow schema cleanup. Pure additive validation. After this lands, malformed workflow.toml files are rejected with clear errors.

NEW RULES:

1. TRIGGER UNIQUENESS. Any state that is the destination of a transition with a non-manual trigger (currently only 'command:start') must have exactly one incoming transition in the entire workflow. No other transition — triggered or manual — may land on that state.

   Rationale: triggers mark a state as freshly ready for an external dispatcher (apm start, apm work, UI dispatcher) to pick up. If another transition can also land on that state, being in the state no longer reliably implies a fresh dispatch is needed. The flag becomes ambiguous.

   Error message names both transitions that violate the rule and identifies the destination state.

2. WORKER_PROFILE SHAPE. If a state declares worker_profile, the value must parse as agent/role where:
   - It contains exactly one '/' separator
   - Both halves are non-empty
   - The role component is not the literal string 'worker' (reserved as process category, not a configured role)

3. COMMAND:START LANDS ON DISPATCH-CAPABLE STATE. Every transition with trigger = 'command:start' must land on a state with worker_profile set. A command:start pointing to a supervisor-owned state has nothing to dispatch.

CONSOLIDATE EXISTING RULES (keep, verify they still fire):
- Terminal states have no outgoing transitions.
- Every non-terminal state is reachable from the new state.
- Workflow has exactly one initial state ('new').

TESTS:
- A workflow where two transitions land on the same state and one is command:start fails validate. Error names both source states.
- A workflow with state.worker_profile = 'claude/worker' fails validate.
- A workflow with state.worker_profile = 'claudecoder' (no slash) fails validate.
- A workflow with state.worker_profile = '/coder' or 'claude/' fails validate.
- A workflow with a command:start to a state without worker_profile fails validate.
- The default workflow (after 071886fc) passes validate.
- This project's .apm/workflow.toml (after 071886fc) passes validate.

OUT OF SCOPE:
- Unifying worker command list (separate ticket).
- Mandatory [workers].default (separate ticket).
- Help text (separate ticket).

REFERENCES:
- apm-core/src/validate.rs or wherever apm validate lives
- apm-core/src/config.rs for the State and Transition struct shapes (after e05c0463)

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
| 2026-05-31T02:57Z | — | new | philippepascal |
