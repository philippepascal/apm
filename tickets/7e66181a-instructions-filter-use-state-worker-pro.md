+++
id = "7e66181a"
title = "Instructions filter: use state.worker_profile for role-scoped output"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7e66181a-instructions-filter-use-state-worker-pro"
created_at = "2026-05-31T01:59:26.775579Z"
updated_at = "2026-05-31T03:03:55.696627Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["1a13dee7"]
+++

## Spec

### Problem

Update apm instructions role filtering to use the new state-level worker_profile rather than the dropped transition.role field.

CURRENT (broken after 1e758cd5 drops transition.role):
- apm-core/src/instructions.rs::format_live_state_machine takes a role argument and filters transitions whose 'derived role' matches.
- The role-command-allowlist also filters the command reference section.

NEW BEHAVIOUR (state-driven):
- For a given role argument R, identify the set of states whose worker_profile has R as the role component (split worker_profile on '/', take the second half). These are the 'R-owned' states.
- Emit every transition out of any R-owned state in the state-machine table. This naturally produces the worker's full lifecycle: for coder, in_progress to implemented and in_progress to blocked are both in. For spec-writer, in_design to specd and in_design to question are both in.
- For no-role argument: keep the existing role-index output from 9ea43165 (already working).

COMMAND REFERENCE FILTERING:
The current code filters the command reference list by role. This part is separately bugged (the coder filter shows every command, which is too broad). Within this ticket, audit role_command_allowlist (or whatever the function is) and decide:
- Either keep the existing per-role allow-list (just verify it is sensible and the coder list is narrowed)
- Or derive the allow-list from the workflow (commands that act on transitions from R-owned states, plus base read commands)
The simpler approach is to fix the static per-role allow-list. Document the choice.

TESTS:
- apm instructions --role coder emits a state-machine table with both ready to in_progress (command:start) AND in_progress to implemented and in_progress to blocked rows. Currently only the first row appears.
- apm instructions --role spec-writer emits transitions out of in_design (specd, question), plus the command:start row into in_design from groomed.
- apm instructions --role coder does NOT include spec-writer's transitions.
- apm instructions --role spec-writer does NOT include coder's transitions.
- The command reference section for coder shows a narrower set than 'every apm command'.
- The command reference section for spec-writer is unchanged in length / contents.

OUT OF SCOPE:
- Schema struct changes (in 1e758cd5)
- Dispatch path (in 1a13dee7)
- The bigger fix that build_system_prompt passes an empty commands slice (it is a separate concern — covered in a non-epic cleanup ticket; do not address here)
- apm validate (in c3f5aa4d)

REFERENCES:
- apm-core/src/instructions.rs::format_live_state_machine, role_command_allowlist
- apm-core/src/config.rs for the new State.worker_profile field

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
| 2026-05-31T01:59Z | — | new | philippepascal |
| 2026-05-31T03:03Z | new | closed | philippepascal |
