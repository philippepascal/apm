+++
id = "1e758cd5"
title = "Schema structs and default workflow rewrite to state-level worker_profile"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1e758cd5-schema-structs-and-default-workflow-rewr"
created_at = "2026-05-31T01:58:27.744519Z"
updated_at = "2026-05-31T03:03:46.975420Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
+++

## Spec

### Problem

Foundation ticket for the workflow schema epic. After this lands, apm parses the new schema and both the default workflow and this project's workflow are in the new shape.

SCOPE:

1. Update apm-core/src/config.rs structs:
   - Add State.worker_profile: Option<String> (form: 'agent/role', e.g. 'claude/coder')
   - Drop State.actionable_by (the agent / supervisor enum)
   - Drop Transition.worker_profile (was Option<String>)
   - Drop Transition.role (the overloaded field)
   - Keep Transition: to, trigger, completion
   - Update Serde derivations to match (deny_unknown_fields on these structs so the old fields cause a parse error rather than silently ignoring them; this makes failed migrations visible)

2. Rewrite apm-core/src/default/workflow.toml to the new schema. Use the lifecycle agreed in 599ed441 (which is being superseded by this epic):
   - new: supervisor-owned; to groomed (manual), to closed (manual)
   - groomed: supervisor-owned; to in_design (command:start), to closed (manual)
   - in_design: worker_profile = 'claude/spec-writer'; to specd (manual), to question (manual)
   - specd: supervisor-owned; to ready (manual), to ammend (manual), to closed (manual)
   - ammend: supervisor-owned; to groomed (manual), to closed (manual)
   - question: supervisor-owned; to groomed (manual), to closed (manual)
   - ready: supervisor-owned; to in_progress (command:start), to ammend (manual), to specd (manual), to closed (manual)
   - in_progress: worker_profile = 'claude/coder'; to implemented (manual, completion = merge), to blocked (manual)
   - blocked: supervisor-owned; to ready (manual), to closed (manual)
   - implemented: supervisor-owned; to closed (manual), to ready (manual), to ammend (manual)
   - merge_failed: supervisor-owned; to implemented (manual, completion = merge), to ready (manual)
   - closed: terminal = true

   Key removals from the previous default workflow: no in_design to ammend, no merge_failed to in_progress.

3. Migrate this project's .apm/workflow.toml to the new schema. Preserve the existing project customisation (completion = merge on merge_failed to implemented).

4. Update every existing unit test in apm-core that constructs a Config / Workflow programmatically. Many tests build minimal Config instances inline; each needs to switch to the new field layout. Pay particular attention to tests in:
   - apm-core/src/config.rs (the impl helper tests)
   - apm-core/src/state.rs
   - apm-core/src/start.rs
   - apm-core/src/sync.rs
   - apm-core/src/instructions.rs

5. Update the apm-core test for default-workflow snapshot if one exists, to reflect the new shape.

OUT OF SCOPE:
- apm validate rules (separate ticket in this epic)
- Dispatch-path consumer updates (separate ticket; reading worker_profile from state instead of transition)
- Instructions filter rewrite (separate ticket)
- CLI help text (separate ticket)
- apm-server / apm-ui surface (separate ticket)
- External-project migration docs (separate ticket)

CONSTRAINTS:
- No backwards-compat parsing of the old schema. Old workflow.toml files must fail to parse with a clear error. The project rule is no backwards-compat shims.
- Do not introduce any new abstractions beyond the schema change. The Transition struct, when stripped of removed fields, should be small.

TESTS:
- A workflow.toml using the new shape parses correctly.
- A workflow.toml with 'role = ...' on a transition fails to parse with a clear field-not-allowed error (deny_unknown_fields).
- A workflow.toml with 'worker_profile = ...' on a transition fails to parse (now state-level only).
- A workflow.toml with 'actionable_by = ...' on a state fails to parse.
- The default workflow.toml parses and matches the agreed lifecycle (one assertion per state: name, worker_profile presence/absence, transition count, terminal flag).

REFERENCES:
- apm-core/src/config.rs
- apm-core/src/default/workflow.toml
- .apm/workflow.toml (project copy)
- Ticket 599ed441 has the full context that motivated this work; it is being superseded by this epic.

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
