+++
id = "599ed441"
title = "Workflow schema: move worker_profile to state, drop transition role, enforce uniqueness"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/599ed441-workflow-schema-move-worker-profile-to-s"
created_at = "2026-05-31T01:53:55.589251Z"
updated_at = "2026-05-31T02:10:38.416823Z"
+++

## Spec

### Problem

GOAL: redesign the workflow schema so the meaning of who owns a state and what worker gets dispatched is encoded once, on the state, not duplicated and overloaded across transitions.

CURRENT PROBLEMS (observed in default workflow and project workflow.toml):

1. Transitions carry a role: field that conflates three different concepts:
   (a) which worker profile to dispatch when trigger = command:start
   (b) which worker role can invoke a given manual transition
   (c) a default placeholder ('worker') used when neither applies, especially on supervisor-only transitions
   The placeholder 'worker' is not a valid role — worker is a process category, not a configured role. coder and spec-writer are configured roles a worker can take. Mixing them in the same field has produced silent misconfiguration (every supervisor transition tagged role: worker, half of coder's lifecycle tagged role: worker instead of role: coder).

2. The instructions filter that scopes apm instructions output to a role uses the transition.role field. With the current overloading, that filter excludes transitions a coder must actually invoke (in_progress to implemented, in_progress to blocked) because they are tagged role: worker, and pollutes outputs with supervisor-only paths.

3. role: worker shows up as a custom role in apm instructions role index. The discovery scan treats it as a discoverable role because it appears in workflow.toml. This is a downstream symptom of the same misconfig.

4. apm validate does not catch role: worker as invalid. No schema enforcement exists for what role: values are allowed on transitions.

5. command:start transitions duplicate the worker_profile field on the transition itself, even though every command:start landing on a given destination always dispatches the same worker. The destination state already implies which worker. The duplication is what enables misconfigurations like a transition saying worker_profile = claude/coder while role = worker.

DESIGN: the destination state owns the worker.

STATE FIELDS:
- name (existing)
- worker_profile (NEW; optional): a string of the form agent/role, for example claude/coder or claude/spec-writer. Presence means this state is agent-owned and the listed profile is the worker dispatched into it. Absence means the state is supervisor-owned (or terminal).
- terminal (existing, where applicable)

DROP from state: actionable_by (was supervisor or agent). Replaced by worker_profile presence/absence.

TRANSITION FIELDS reduce to:
- to (existing)
- trigger (existing; manual or command:start; possibly future kinds)
- completion (existing, optional; merge / pr / pr_or_epic_merge / none)

DROP from transition:
- worker_profile (now derived from the destination state)
- role (deleted entirely; the concept it was approximating is now state-level via worker_profile)

DISPATCH SEMANTICS:
- When a transition fires with trigger = command:start, the dispatcher resolves the destination state's worker_profile and spawns that worker. If the destination has no worker_profile, the transition is invalid (validate rule below).
- When a transition fires with trigger = manual (apm state command), no dispatch happens regardless of the destination's worker_profile. This is critical for cases like supervisor returning a ticket to an agent-owned state without re-dispatching; with the new rule below those manual-into-agent-owned paths are disallowed entirely, but the semantics are preserved for any edge case.

DEFAULT WORKFLOW CORRECTIONS (apply to apm-core/src/default/workflow.toml AND audit/migrate .apm/workflow.toml in this project and in syn):

REMOVE these transitions:
- in_design to ammend (spec-writer never goes to ammend mid-flow; supervisor moves specd to ammend)
- merge_failed to in_progress (merge_failed recovers only to ready or implemented)

The full lifecycle becomes:
- new: supervisor-owned. Transitions: to groomed (manual), to closed (manual)
- groomed: supervisor-owned. Transitions: to in_design (command:start), to closed (manual)
- in_design: worker_profile = claude/spec-writer. Transitions: to specd (manual), to question (manual)
- specd: supervisor-owned. Transitions: to ready (manual), to ammend (manual), to closed (manual)
- ammend: supervisor-owned. Transitions: to groomed (manual), to closed (manual)
- question: supervisor-owned. Transitions: to groomed (manual), to closed (manual)
- ready: worker_profile = claude/coder? NO — ready is supervisor-owned and dispatch happens on entry to in_progress. Transitions: to in_progress (command:start), to ammend (manual), to specd (manual), to closed (manual)
- in_progress: worker_profile = claude/coder. Transitions: to implemented (manual, completion = merge), to blocked (manual)
- blocked: supervisor-owned. Transitions: to ready (manual), to closed (manual)
- implemented: supervisor-owned. Transitions: to closed (manual), to ready (manual), to ammend (manual)
- merge_failed: supervisor-owned. Transitions: to implemented (manual, completion = merge), to ready (manual)
- closed: terminal

(Note: the project's local workflow.toml currently uses completion = merge on the merge_failed to implemented transition; preserve that.)

NEW VALIDATE RULE (the trigger-uniqueness rule):

Any state that is the destination of a transition with a non-manual trigger (currently only command:start) must have exactly one incoming transition in the entire workflow. No other transition may land on that state, whether triggered or manual.

Rationale: a trigger marks a state as freshly ready for an external dispatcher (apm start, apm work, UI dispatcher) to pick up. If another transition can also land on that state, being in the state no longer reliably implies a fresh dispatch is needed; the trigger flag is then ambiguous. The rule enforces that triggered entry is the only entry.

Consequences for the workflow above:
- in_design is reached only from groomed (command:start). ammend has no direct path; it must go ammend to groomed to in_design.
- in_progress is reached only from ready (command:start). implemented to in_progress goes via implemented to ready to in_progress (supervisor sends back to ready, then someone starts it fresh).

OTHER VALIDATE RULES (consolidate; some may already exist):
- worker_profile on a state, if present, has the form <agent>/<role> where the role file exists at .apm/agents/<agent>/apm.<role>.md or falls back through the cascade.
- worker_profile value must not include 'worker' as the role component.
- Terminal states have no outgoing transitions.
- Every non-terminal state must be reachable from new.
- Every command:start transition must land on a state that has worker_profile set. (A command:start with a dispatch-less destination is meaningless.)

CONSUMER UPDATES:

1. apm-core/src/instructions.rs::format_live_state_machine — replace transition.role filtering with state-level filtering. For a given role argument:
   - Identify all states with worker_profile whose role component equals the argument.
   - Emit all outgoing transitions from those states in the table.
   - This correctly captures the worker's full lifecycle (start, finish, escape, etc.).
   For no-role argument, behaviour remains the role-index (already implemented in 9ea43165).

2. apm-core/src/start.rs::resolve_worker_profile and the dispatch path — read worker_profile from the destination state's frontmatter, not from the transition. The cascade order (state.worker_profile → workers.default → built-in fallback) replaces the current (transition.worker_profile → workers.default → fallback).

3. apm-core/src/start.rs::resolve_for_diagnostic (apm agents resolve) — same change. When determining what would dispatch for a ticket in state X, look at the command:start transition out of X, take its destination, read that destination's worker_profile. Provenance source label should read 'workflow.toml state <name>.worker_profile' (or 'workers.default' / 'built-in fallback' as before).

4. apm validate — implement the new rules above. Generate clear error messages: 'state <X> is reachable via a triggered transition from <Y> and via a manual transition from <Z>; triggered states must have exactly one incoming transition' etc.

5. apm-core/src/config.rs — update the State struct and Transition struct to drop the removed fields (transition.worker_profile, transition.role, state.actionable_by) and add State.worker_profile. Deserialize accordingly.

6. The helper config.implementation_state_ids() currently derives implementation states from transition shape (coder start + merge completion). After the redesign, the same set can be derived more simply: an implementation state is one with worker_profile set whose role component is the coder role, plus any state that is the destination of a merge-completion transition. Keep the same external semantics; simplify the internal logic.

7. apm-core/src/recovery.rs — review whether the merge-failure classification needs updating. Should still work since it keys off completion strategies on transitions.

8. apm-core/src/sync.rs — review uses of role/worker_profile. The implementation-state set comes from config helper; should continue to work.

MIGRATION:

This is a breaking schema change for any .apm/workflow.toml in the wild. Two affected projects: this repo (apm/.apm/workflow.toml) and syn (the user's other project). apm init writes the default workflow on initialisation. Existing workflow.toml files must be migrated.

Provide a one-off migration helper or document the migration:
- For each state with actionable_by = agent: identify the worker_profile from incoming command:start transitions or from the role: field of outgoing transitions tagged with a configured role. Add worker_profile to the state.
- Drop transition.worker_profile and transition.role from every transition.
- Remove in_design to ammend and merge_failed to in_progress.
- Run apm validate; resolve errors.

Whether to write a migration command (apm migrate-workflow or similar) or document the manual edits is the spec-writer's call. Probably document for now and ask supervisor whether automated migration is wanted.

OUT OF SCOPE:
- The other apm prompt / apm instructions bugs (build_system_prompt passes empty commands, prompt --help text is stale, coder Command Reference filter is too broad, instructions one-line summary still mentions shell discipline). Those are separate tickets.
- Changes to apm-server / apm-ui beyond what is required by the schema change.
- Changes to the worker spawn machinery beyond reading worker_profile from a different location.
- Adding new state or new trigger kinds. The change is scoped to schema cleanup and rule enforcement on the existing model.

TESTS:
- Schema parsing: a workflow.toml using the new shape (state-level worker_profile, transitions with only to/trigger/completion) parses correctly.
- Schema rejection: a workflow.toml with role: worker on any transition fails validate with a clear error.
- Schema rejection: a workflow.toml with role: anything on a transition fails (or warns) — the field no longer exists.
- Schema rejection: a workflow.toml where two transitions land on the same state and at least one has trigger = command:start fails validate.
- Schema rejection: a command:start transition pointing to a state without worker_profile fails validate.
- Schema acceptance: the default workflow (after this change) passes all rules.
- Dispatch: resolve_for_diagnostic on a ticket in groomed reports the spec-writer based on in_design.worker_profile, not from a transition field.
- Instructions filter: apm instructions --role coder emits all transitions out of states with worker_profile = claude/coder (i.e., the coder's full lifecycle: in_progress to implemented and in_progress to blocked).
- Instructions filter: apm instructions --role spec-writer emits transitions out of states with worker_profile = claude/spec-writer.

REFERENCES:
- apm-core/src/default/workflow.toml — current default
- .apm/workflow.toml — current project override (note the completion = merge customisation on merge_failed to implemented)
- apm-core/src/config.rs — State and Transition structs, implementation_state_ids helper
- apm-core/src/instructions.rs — format_live_state_machine, role filter logic
- apm-core/src/start.rs — build_system_prompt, resolve_for_diagnostic, dispatch path
- apm-core/src/validate.rs (or wherever apm validate lives)
- apm-core/src/sync.rs — implementation_state_ids consumer

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
| 2026-05-31T01:53Z | — | new | philippepascal |
| 2026-05-31T02:10Z | new | closed | philippepascal |
