+++
id = "28ac0f43"
title = "Add state.worker_profile; dispatch reads it (transition fallback retained)"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/28ac0f43-add-state-worker-profile-dispatch-reads-"
created_at = "2026-05-31T02:56:42.034762Z"
updated_at = "2026-05-31T02:56:42.034762Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["f7340b57"]
+++

## Spec

### Problem

STEP 2 of the incremental workflow schema cleanup. After this lands, dispatch resolution and the instructions state-machine filter both prefer state.worker_profile, with the old transition.worker_profile still functional as a fallback. The system stays fully working.

SCOPE:

1. Update apm-core/src/config.rs::StateConfig:
   - Add a new field worker_profile: Option<String>. Format: agent/role.
   - Document the field in the doc comment: when set, the state is owned by that worker profile. The dispatcher reads this on transitions into the state.

2. Update apm-core/src/default/workflow.toml:
   - Add worker_profile = 'claude/spec-writer' on the in_design state.
   - Add worker_profile = 'claude/coder' on the in_progress state.

3. Migrate this project's .apm/workflow.toml the same way.

4. Update apm-core/src/start.rs dispatch resolution (run, run_next, spawn_next_worker, resolve_for_diagnostic):
   - When determining the worker profile to dispatch, look at the destination state's worker_profile FIRST.
   - Fall back to the firing transition's worker_profile if the destination state has none. (Backwards compatibility during the staged migration.)
   - Then fall back to config.workers.default.
   - Then fall back to the built-in 'claude/coder'.

5. Update apm-core/src/instructions.rs::format_live_state_machine — the per-role filter:
   - Change the filter from derive_transition_role(transition) to a state-based lookup: identify states whose worker_profile role component equals the requested role; emit all transitions out of those states.
   - Keep derive_transition_role for now if it still has callers; if it has no callers after this change, delete it.

6. Update apm-core/src/agents.rs scan logic — the bit that walks worker_profile transitions to determine referenced agents. Update to walk worker_profile states first, transitions as fallback.

7. Update apm-core/src/config.rs::implementation_state_ids — the helper that returns implementation state ids. The 'coder start' part should derive from state worker_profile presence; keep the 'merge completion' part on transitions. Result set must remain semantically equivalent on the default workflow.

OUT OF SCOPE:
- Dropping transition.worker_profile (next ticket).
- Removing the built-in 'claude/coder' fallback (later ticket about making [workers].default mandatory).
- Updating workflow transitions or removing bad ones (later ticket).
- Trigger uniqueness validate (later ticket).
- Help text (later ticket).
- Server / UI surfaces (later ticket).

TESTS:
- A workflow.toml with state.worker_profile = 'claude/coder' parses correctly.
- Dispatching a worker for a ticket transitioning from ready to in_progress reads worker_profile from in_progress state (not from the transition). Add a test that explicitly removes the transition.worker_profile and confirms the dispatch still resolves to claude/coder via the state.
- A workflow.toml that has BOTH state.worker_profile and transition.worker_profile prefers the state value.
- A workflow.toml that has only transition.worker_profile (legacy shape) still dispatches correctly via the transition fallback path.
- The instructions filter for role = coder includes all transitions out of in_progress (the coder's full lifecycle: implemented, blocked, and any other), not only the command:start row.
- resolve_for_diagnostic provenance correctly names the state when worker_profile was sourced from the state (label: 'workflow.toml state <name>.worker_profile').

REFERENCES:
- apm-core/src/config.rs
- apm-core/src/start.rs (the four resolution sites and resolve_for_diagnostic)
- apm-core/src/instructions.rs (format_live_state_machine, derive_transition_role)
- apm-core/src/agents.rs (the scan logic)
- apm-core/src/default/workflow.toml
- .apm/workflow.toml

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
| 2026-05-31T02:56Z | — | new | philippepascal |
