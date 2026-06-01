+++
id = "e20488b3"
title = "Implicit closed transition: drop explicit close entries from workflow.toml"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e20488b3-implicit-closed-transition-drop-explicit"
created_at = "2026-06-01T07:14:21.218455Z"
updated_at = "2026-06-01T07:14:21.218455Z"
+++

## Spec

### Problem

GOAL: stop listing explicit close transitions per state in workflow.toml. Instead, the state machine treats close as a cross-cutting supervisor escape hatch: any non-terminal state can transition to closed via manual trigger, regardless of whether the workflow.toml lists it. This drops 10 redundant transition entries per workflow.toml file and clarifies that close is a system action, not part of any state's lifecycle.

PROBLEM:

Counting the current default workflow.toml: 10 states each carry a transition with to = closed, trigger = manual, outcome = cancelled. All identical shape. They are pure noise — they encode no semantic information beyond what the system already knows (closed is terminal, supervisor can always end a ticket). Every new state added to the workflow has to remember to include the close transition. Conflating close with lifecycle transitions makes the workflow.toml harder to read.

DESIGN:

Encode the implicit rule in the state machine: the supervisor can transition any non-terminal state to a terminal state (currently only closed) via manual trigger, without that transition being listed in workflow.toml.

Specifically:
- apm-core/src/state.rs (or wherever transitions are validated): when looking up a transition for apm state <id> <target>, if the target is a terminal state and the source is a non-terminal state, accept the transition even if it is not in the source state's transitions list.
- This applies to ALL non-terminal states including agent-owned states like in_progress and in_design (option B from the discussion that motivated this ticket). Supervisor can force-close an in-progress ticket; preserves today's behaviour.

WORKFLOW.TOML CHANGES:

Remove every transition with to = closed from both files:
- apm-core/src/default/workflow.toml: 10 entries to remove
- .apm/workflow.toml (this project): 10 entries to remove

The closed state itself stays (with terminal = true and no outgoing transitions).

VALIDATE RULE:

Add to apm-core/src/validate.rs: explicit transitions to a terminal state are no longer allowed. They are redundant with the implicit rule and would cause confusion. Error message names the state and the redundant transition.

This implicitly removes the trigger-uniqueness rule consideration for closed (closed is no longer a destination listed in any transition). The relaxed rule (no mix of triggered and manual on the same destination) continues to apply to in_design and in_progress.

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- apm state <id> closed works from every non-terminal state (new, groomed, question, specd, ammend, in_design, ready, in_progress, blocked, implemented, merge_failed) without workflow.toml listing the transition
- apm state <id> closed fails from closed (already terminal)
- A workflow.toml that lists an explicit transition to a terminal state fails validate with a clear error
- The default workflow.toml has zero transitions to closed
- All existing tests that exercise apm state ... closed continue to pass
- cargo test --workspace passes

OUT OF SCOPE:

- Adding new terminal states (abandoned, wontfix, archived). The implicit rule will work for them when added later, but this ticket only introduces the rule for the existing closed state.
- Changing who can invoke close (still anyone with apm CLI access; no permission system added).
- Worker cleanup when force-closing an agent-owned in-progress state (worker processes may need separate handling; out of scope for this ticket).
- Changes to apm-server / apm-ui beyond the underlying state machine behaviour change.

REFERENCES:
- apm-core/src/state.rs (transition validation)
- apm-core/src/validate.rs (new rule)
- apm-core/src/default/workflow.toml
- .apm/workflow.toml
- apm-core/src/config.rs (terminal flag on StateConfig)
- Discussion in conversation history: option B (any non-terminal state, including agent-owned) was selected to preserve current behaviour of allowing supervisor to force-close in-progress tickets

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
| 2026-06-01T07:14Z | — | new | philippepascal |
