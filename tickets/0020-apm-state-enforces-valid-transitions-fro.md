+++
id = 20
title = "apm state enforces valid transitions from state machine config"
state = "specd"
priority = 5
effort = 4
risk = 3
branch = "ticket/0020-apm-state-enforces-valid-transitions-fro"
created = "2026-03-26"
updated = "2026-03-26"
+++

## Spec

### Problem

`apm state <id> <state>` validates that the target state exists in
`[[workflow.states]]` (#14), but does not check whether the transition from the
current state to the target state is permitted. The state machine in `apm.toml`
can define `[[workflow.states.transitions]]` entries that restrict which
`from → to` moves are legal for a given actor. Ignoring these rules means the
enforcement in #14 is incomplete: a ticket can jump from `new` directly to
`closed` without passing through the defined workflow.

### Acceptance criteria

- [ ] When `[[workflow.states]]` entries include `[[transitions]]`, `apm state` checks that a transition from the current state to the requested state exists
- [ ] If no transition is found: exit non-zero with message: `no transition from "X" to "Y" — valid transitions from "X": Z, W`
- [ ] If the current state has no transitions defined (empty list), the transition is allowed — empty list means unconstrained
- [ ] If `[[workflow.states]]` is empty (no config), all transitions are allowed (safe fallback, same as today)
- [ ] No file is modified if the transition is rejected
- [ ] All existing tests continue to pass

### Out of scope

- Actor-based enforcement (`actor = "agent"` vs `actor = "supervisor"`) — separate ticket
- Enforcing preconditions (e.g. spec must be complete before `ready`)

### Approach

In `cmd/state.rs`, after the existing state-exists check, look up the
`StateConfig` for the current ticket state. If `transitions` is non-empty, check
whether any entry has `to == new_state`. If none match, bail with the error
message listing valid `to` targets for the current state.

`StateConfig` already has a `transitions: Vec<TransitionConfig>` field in
`apm-core/src/config.rs` — no model changes needed.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | agent | new → specd | |
| 2026-03-26 | agent | ready → closed | |
