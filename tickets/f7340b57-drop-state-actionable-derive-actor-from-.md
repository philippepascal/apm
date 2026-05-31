+++
id = "f7340b57"
title = "Drop state.actionable; derive actor from outgoing triggers"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7340b57-drop-state-actionable-derive-actor-from-"
created_at = "2026-05-31T02:56:19.482471Z"
updated_at = "2026-05-31T02:56:19.482471Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

STEP 1 of the incremental workflow schema cleanup. After this lands, the system has identical behaviour but the actionable field is gone.

PROBLEM:
- StateConfig has a Vec<String> field 'actionable' with values 'agent', 'supervisor', 'engineer', 'any'.
- In practice only 'agent' and 'supervisor' are used across apm and syn workflows. 'engineer' and 'any' are unused.
- The information actionable encodes is derivable: a state is agent-actionable IF it has an outgoing transition with trigger = command:start, otherwise it is supervisor-actionable.
- Keeping the explicit field invites future inconsistency (manual edits where the field disagrees with the trigger shape).

SCOPE:

1. Update apm-core/src/config.rs:
   - Drop the State.actionable field from StateConfig.
   - Drop the doc comment that references 'engineer' and 'any'.
   - Add deny_unknown_fields on StateConfig so leftover 'actionable = [...]' lines in workflow.toml will fail to parse with a clear error.

2. Update apm-core/src/config.rs::actionable_states_for(actor) — the consumer that returns the list of state ids for a given actor. New implementation:
   - For actor = 'agent': return state ids that have at least one outgoing transition with trigger = 'command:start'.
   - For actor = 'supervisor': return state ids that are not terminal and do NOT have a command:start out (only manual transitions).
   - For any other actor: return empty (preserves binary semantic without 'engineer' / 'any').

3. Rewrite apm-core/src/default/workflow.toml to remove every 'actionable = [...]' line.

4. Migrate this project's .apm/workflow.toml to remove every 'actionable = [...]' line.

5. Update every unit test in apm-core that constructs a Config / Workflow programmatically with an actionable field. Remove the field from the test fixtures.

6. Audit the codebase for any other reference to state.actionable. Update or remove.

OUT OF SCOPE:
- Worker_profile changes (later ticket).
- Workflow transitions changes (later ticket).
- Validate rules (later ticket).
- Help text updates (later ticket).
- Command reference list changes (later ticket).

TESTS:
- A workflow.toml using the new shape (no actionable field) parses correctly.
- A workflow.toml with 'actionable = ["agent"]' on any state fails to parse with a clear error pointing at the unknown field.
- apm next and apm list --actionable behave identically on the default workflow before and after this change. Recommend a behavioural test that creates tickets in various states and asserts the same returned set.

REFERENCES:
- apm-core/src/config.rs (StateConfig, actionable_states_for)
- apm-core/src/default/workflow.toml
- .apm/workflow.toml (project)
- Discussion in conversation history: the old epic a42eceea has the original (now-rejected) design that kept the field

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
