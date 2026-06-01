+++
id = "e20488b3"
title = "Implicit closed transition: drop explicit close entries from workflow.toml"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e20488b3-implicit-closed-transition-drop-explicit"
created_at = "2026-06-01T07:14:21.218455Z"
updated_at = "2026-06-01T07:16:16.212088Z"
+++

## Spec

### Problem

Every non-terminal state in the default workflow lists an explicit `[[workflow.states.transitions]]` block with `to = "closed"`, `trigger = "manual"`, and `outcome = "cancelled"`. Across the 10 non-terminal states in a standard workflow, that is 10 identical entries that encode no workflow-specific information — pure repetition. Worse, any new state added to the workflow must remember to include this boilerplate, creating a maintenance trap. Conflating close with normal lifecycle transitions also makes workflow.toml harder to read: a reader cannot distinguish "this state has five lifecycle options" from "this state has four lifecycle options plus the universal emergency exit."

The fix is to encode the rule where it belongs — in the state machine itself — and remove it from every workflow.toml. The `transition()` function in `apm-core/src/state.rs` already skips the explicit-transition check when the target is terminal (the `else if !target_is_terminal` branch at lines 57–82). Making this the documented contract, adding the inverse guard (reject transitions *from* a terminal state), and forbidding explicit terminal transitions in workflow.toml files completes the behaviour. The 10 boilerplate entries per workflow.toml are deleted, and `apm validate` enforces the new invariant on all future configs.

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
| 2026-06-01T07:16Z | new | groomed | philippepascal |
| 2026-06-01T07:16Z | groomed | in_design | philippepascal |