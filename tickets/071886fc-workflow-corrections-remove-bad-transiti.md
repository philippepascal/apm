+++
id = "071886fc"
title = "Workflow corrections: remove bad transitions, restructure ammend path"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/071886fc-workflow-corrections-remove-bad-transiti"
created_at = "2026-05-31T02:57:20.412089Z"
updated_at = "2026-05-31T07:04:35.474316Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463"]
+++

## Spec

### Problem

STEP 4 of the incremental workflow schema cleanup. Pure workflow.toml content change. No code change needed (the state machine accepts whatever transitions are defined).

PROBLEM: the default workflow has three transitions that should not exist:
- in_design to ammend: a spec-writer mid-flow does not jump to ammend; ammend is a supervisor-initiated state from specd.
- merge_failed to in_progress: a merge failure recovers to ready (escalate) or implemented (retry merge); never back to in_progress.
- ammend to in_design via command:start: this makes in_design have TWO triggered entries (groomed and ammend), violating the trigger-uniqueness rule planned for step 5. The clean path is ammend to groomed (manual), then groomed to in_design (command:start).

SCOPE:

1. apm-core/src/default/workflow.toml:
   - Remove the transition from in_design with to = 'ammend'.
   - Remove the transition from merge_failed with to = 'in_progress'.
   - On the ammend state: change the transition with to = 'in_design' (trigger command:start) to to = 'groomed' (trigger manual).

2. .apm/workflow.toml (this project): apply the same three changes.

3. Update any unit / integration tests that exercised these specific transitions. They were probably the inverse of the desired invariant; the tests can be deleted or repurposed.

4. Update apm-core/src/default/agents/claude/apm.spec-writer.md and apm-core/src/default/agents/claude/apm.coder.md (and their .apm/ project copies) if they reference the removed transitions in their guidance prose. Spec-writers should be told 'when blocked from in_design, transition to question; do not transition to ammend.'

OUT OF SCOPE:
- The trigger-uniqueness validate rule (next ticket, which depends on this correction).
- Other workflow content changes.
- Help text or docs sweep (later ticket).

TESTS:
- The default workflow parses (already covered by the earlier tickets).
- apm next + apm list --actionable behave correctly with the new state graph. Specifically: ammend is supervisor-actionable (no command:start out), groomed is agent-actionable.
- A ticket can go specd to ammend (manual) to groomed (manual) to in_design (command:start) and back to specd (manual).

REFERENCES:
- apm-core/src/default/workflow.toml
- .apm/workflow.toml
- apm-core/src/default/agents/claude/apm.spec-writer.md
- apm-core/src/default/agents/claude/apm.coder.md

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
