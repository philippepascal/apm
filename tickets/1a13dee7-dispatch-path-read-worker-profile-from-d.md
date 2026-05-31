+++
id = "1a13dee7"
title = "Dispatch path: read worker_profile from destination state, not transition"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1a13dee7-dispatch-path-read-worker-profile-from-d"
created_at = "2026-05-31T01:59:08.692474Z"
updated_at = "2026-05-31T03:03:52.961874Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["1e758cd5"]
+++

## Spec

### Problem

Update the dispatch and resolution code in apm-core to read worker_profile from the destination state's frontmatter, not from the transition.

CONSUMERS TO UPDATE:

1. apm-core/src/start.rs::resolve_worker_profile (or the inline resolution in run / run_next / spawn_next_worker). The current cascade is approximately:
   - transition.worker_profile (which no longer exists after 1e758cd5)
   - config.workers.default
   - built-in fallback ('claude/coder')

   The new cascade is:
   - destination_state.worker_profile (where destination = the to-state of the command:start transition being fired)
   - config.workers.default
   - built-in fallback

2. apm-core/src/start.rs::resolve_for_diagnostic (the 'apm agents resolve' backend). Update the provenance string accordingly. Where the current code outputs 'workflow.toml transition <from> → <to>', the new label is something like 'workflow.toml state <to>.worker_profile'. Carry the original transition_label as a separate piece of metadata.

3. apm-core/src/config.rs::implementation_state_ids. Current implementation derives the set from transitions that are command:start with a non-spec-writer profile OR transitions with completion = Pr / Merge / PrOrEpicMerge. With the new schema:
   - The 'coder start' part becomes 'states with worker_profile whose role component is not spec-writer that are reached via command:start'
   - The 'merge completion' part is unchanged (still on transitions)

   Keep the set semantically equivalent. Tests must still pass.

4. apm-core/src/sync.rs uses implementation_state_ids and terminal_state_ids. Verify those callers still get the same logical set.

5. apm-core/src/recovery.rs (is_merge_failure_state, classify_recovery_options). Review for any reference to transition.worker_profile or transition.role; update if present.

6. Any other code that reads transition.worker_profile or transition.role — grep for it.

TESTS TO UPDATE / ADD:
- resolve_for_diagnostic on a ticket in 'groomed' state reports the spec-writer based on in_design.worker_profile. The provenance string identifies in_design.worker_profile, not a transition field.
- resolve_for_diagnostic happy-path, override, manifest-absent, non-dispatchable cases (the four existing tests from 36b6f742) still pass.
- build_system_prompt still produces a coherent system prompt when dispatched for a coder role.
- implementation_state_ids on the new default workflow returns the expected set (in_design and in_progress, conceptually — verify against the lifecycle).

OUT OF SCOPE:
- Schema struct changes (in 1e758cd5).
- apm validate rules (in c3f5aa4d).
- Instructions filter (separate ticket — reads from same place but the filter logic is separate).
- CLI help text and apm-server / apm-ui (separate tickets).

CONSTRAINTS:
- Do not change the public signature of resolve_for_diagnostic if avoidable. The AgentDiagnostic struct fields can stay the same; just the source labels and the resolution logic change.
- The cascade order (state worker_profile, then workers.default, then built-in) must be preserved.

REFERENCES:
- apm-core/src/start.rs (resolve_worker_profile, resolve_for_diagnostic, run/run_next/spawn_next_worker)
- apm-core/src/config.rs (implementation_state_ids, terminal_state_ids)
- apm-core/src/sync.rs (consumers)
- apm-core/src/recovery.rs

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
