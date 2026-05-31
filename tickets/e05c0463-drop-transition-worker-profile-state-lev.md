+++
id = "e05c0463"
title = "Drop transition.worker_profile (state-level is the only source)"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e05c0463-drop-transition-worker-profile-state-lev"
created_at = "2026-05-31T02:57:03.550888Z"
updated_at = "2026-05-31T07:04:32.935016Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["28ac0f43"]
+++

## Spec

### Problem

STEP 3 of the incremental workflow schema cleanup. After 28ac0f43 lands, both state and transition can carry worker_profile; this ticket removes the transition-level field so state is the only source. After this lands, old workflow.toml files (with transition.worker_profile) fail to parse with a clear error.

SCOPE:

1. Update apm-core/src/config.rs::TransitionConfig:
   - Drop the worker_profile: Option<String> field.
   - Add deny_unknown_fields on TransitionConfig so any old workflow.toml with 'worker_profile = ...' under [[workflow.states.transitions]] fails parsing.

2. Update the parse-error pathway:
   - When the workflow.toml parse fails because of an unknown field (deny_unknown_fields), the error message must clearly identify the offending field and reference the migration steps (a brief one-liner is enough; full migration docs not required by this ticket).
   - Use anyhow::Context or thiserror to add the friendly message.

3. Remove the transition.worker_profile fallback added in 28ac0f43:
   - In start.rs dispatch resolution sites, drop the second-tier fallback (was: state.worker_profile, then transition.worker_profile, then workers.default, then built-in). New: state.worker_profile, then workers.default, then built-in.
   - Update resolve_for_diagnostic similarly.

4. Update apm-core/src/default/workflow.toml — remove every remaining transition.worker_profile line (28ac0f43 should have added state-level equivalents; verify nothing references the transition field).

5. Migrate this project's .apm/workflow.toml the same way.

6. Update apm-core/src/agents.rs scan logic — drop the transition-fallback walk if 28ac0f43 left it.

7. If derive_transition_role still exists in instructions.rs and has no consumers, delete it.

OUT OF SCOPE:
- The remaining built-in 'claude/coder' fallback (next ticket: mandatory workers.default).
- Workflow transition corrections (later ticket).
- Validate rules (later ticket).

TESTS:
- A workflow.toml with worker_profile under any transition fails to parse with a clear error.
- A workflow.toml with all worker_profile values at state level parses correctly.
- Dispatch resolution after this ticket reads ONLY from state.worker_profile and workers.default.
- All existing tests pass (no tests should depend on transition-level worker_profile by this point).

REFERENCES:
- apm-core/src/config.rs (TransitionConfig)
- apm-core/src/start.rs (resolution sites)
- apm-core/src/default/workflow.toml
- .apm/workflow.toml
- apm-core/src/agents.rs

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
| 2026-05-31T07:04Z | new | groomed | philippepascal |
