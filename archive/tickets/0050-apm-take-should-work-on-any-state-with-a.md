+++
id = 50
title = "apm take should work on any state with an assigned agent"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "claude-0328-1430-a4f2"
agent = "claude-0328-1430-a4f2"
branch = "ticket/0050-apm-take-should-work-on-any-state-with-a"
created_at = "2026-03-28T22:11:45.364790Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm take` only accepts tickets in `in_progress` or `implemented` state. Any
other state (including `new`, `ammend`, `blocked`, `ready`) produces:

```
ticket #N is in state "..." — take requires in_progress or implemented
```

The restriction is wrong. The purpose of `apm take` is to reassign the `agent`
field when a different agent needs to continue work. The relevant condition is
not the state but whether an agent is currently assigned. Any state where
`agent` is set can be taken over — for example:

- `ammend`: original agent is gone; new agent wants to address the amendments
- `blocked`: ticket needs to be reassigned before unblocking
- `ready` (with agent set): agent claimed it but hasn't started yet

Conversely, a ticket with no agent set does not need `take` — it is unclaimed
and `apm start` is the correct entry point.

### Acceptance criteria

- [x] `apm take <id>` succeeds on any ticket where `agent` is set, regardless of state
- [x] `apm take <id>` fails with a clear error when `agent` is not set (use `apm start` instead)
- [x] The existing `handoff` history entry is still appended on success
- [x] All existing tests pass

### Out of scope

- Changing which states `apm start` accepts (still requires `ready`)
- Adding any new states

### Approach

In `apm/src/cmd/take.rs`, replace the state allowlist check:

```rust
// Before
if fm.state != "in_progress" && fm.state != "implemented" {
    bail!("ticket #{id} is in state {:?} — take requires in_progress or implemented", fm.state);
}

// After
if fm.agent.is_none() {
    bail!("ticket #{id} has no assigned agent — use `apm start` to claim it");
}
```

No other changes needed.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T22:11Z | — | new | claude-0328-1430-a4f2 |
| 2026-03-28T22:17Z | new | specd | claude-0328-1430-a4f2 |
| 2026-03-28T22:23Z | specd | ready | apm |
| 2026-03-28T22:25Z | ready | in_progress | claude-0328-1430-a4f2 |
| 2026-03-28T22:27Z | in_progress | implemented | claude-0328-1430-a4f2 |
| 2026-03-28T23:07Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |