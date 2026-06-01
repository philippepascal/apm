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

- [ ] `apm state <id> closed` succeeds from `new` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `groomed` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `question` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `specd` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `ammend` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `in_design` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `ready` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `in_progress` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `blocked` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `implemented` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` succeeds from `merge_failed` without a `to = "closed"` entry in workflow.toml
- [ ] `apm state <id> closed` fails with a clear error when the ticket is already in state `closed`
- [ ] `apm validate` rejects a workflow.toml that contains an explicit `to = "closed"` (or any terminal state) transition, and the error message names both the source state and the terminal target
- [ ] `apm-core/src/default/workflow.toml` contains zero `[[workflow.states.transitions]]` blocks with `to = "closed"`
- [ ] `cargo test --workspace` passes

### Out of scope

- Adding new terminal states (`abandoned`, `wontfix`, `archived`). The implicit rule will work for them when added, but this ticket only introduces the rule for the existing `closed` state.
- Changing who can invoke close. Any user with `apm` CLI access can still run `apm state <id> closed`; no permission system is added.
- Worker process cleanup when force-closing an `in_progress` or `in_design` ticket. The worktree and any running agent process are not affected.
- Changes to `apm-server` or `apm-ui` beyond what is required to keep existing tests passing. The UI may no longer list `closed` as a transition option; that is intentional and not a regression.
- Changes to `available_transitions` or `compute_valid_transitions` in `state.rs`. The implicit close is a supervisor escape hatch, intentionally absent from those lists.

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