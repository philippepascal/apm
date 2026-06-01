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

#### 1. apm-core/src/state.rs — Block transitions from terminal source states

In `transition()`, after `old_state` is set (line 49) and before the completion-strategy lookup, add:

```rust
if !force {
    if let Some(src) = config.workflow.states.iter().find(|s| s.id == old_state) {
        if src.terminal {
            bail!("ticket {:?} is in terminal state {:?}; no further transitions are allowed", id, old_state);
        }
    }
}
```

The implicit allow for terminal *targets* is already in place: the `else if !target_is_terminal` branch (lines 57–82) falls through to `(CompletionStrategy::None, None)` when `target_is_terminal = true`, bypassing the explicit-transition check. No further change to transition lookup logic is needed.

#### 2. apm-core/src/validate.rs — New rule + update two existing checks

In `validate_config_no_agents()`, inside the `for transition in &state.transitions` loop (near line 381):

**Add before the "target must exist" check:**
```rust
if terminal_ids.contains(transition.to.as_str()) {
    errors.push(format!(
        "config: state.{}.transition({}) — explicit transitions to terminal states are not \
         allowed; {} is always reachable as a supervisor close action",
        state.id, transition.to, transition.to
    ));
}
```

**Update the "target must exist" check (currently line 384)** — replace the `!= "closed"` hard-code with the general terminal check:
```rust
// Before:
if transition.to != "closed" && !state_ids.contains(transition.to.as_str()) {
// After:
if !terminal_ids.contains(transition.to.as_str()) && !state_ids.contains(transition.to.as_str()) {
```

**Remove the dead-end error** (currently lines 373–380): with the implicit close rule, any non-terminal state can always reach `closed`, so "tickets will be stranded" is no longer accurate. The existing BFS reachability warning in `validate_warnings` already catches workflows where no success outcome is reachable.

#### 3. Workflow.toml files — Remove 10 closed-transition blocks each

From `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`, delete every block of the form:
```toml
[[workflow.states.transitions]]
to      = "closed"
trigger = "manual"
outcome = "cancelled"
```

This affects `new`, `groomed`, `question`, `specd`, `ammend`, `in_design`, `ready`, `in_progress`, `blocked`, and `implemented` — 10 entries per file, 20 total. The `closed` state declaration (with `terminal = true`) is kept unchanged.

#### 4. Tests

**validate.rs — update configs in broken tests:** remove any `to = "closed"` entries from inline test TOML configs that call `validate_config`. The `correct_config_passes` test (line 1210) is the primary one; give `in_progress` a non-terminal transition to replace the closed entry.

**validate.rs — add new test** `explicit_terminal_transition_rejected`: verify a config with `to = "closed"` in a non-terminal state's transitions produces an error naming the source state and terminal target.

**state.rs — update helper** `config_with_transitions()` (line 407): remove the `to = "closed"` entry; update `compute_valid_transitions_returns_expected_options` to expect one transition instead of two.

**apm/tests/integration.rs — add two tests:**
- Implicit close succeeds from a non-terminal state without the transition listed.
- Close from terminal state fails with the expected error message.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-01T07:14Z | — | new | philippepascal |
| 2026-06-01T07:16Z | new | groomed | philippepascal |
| 2026-06-01T07:16Z | groomed | in_design | philippepascal |