+++
id = "8b3fc691"
title = "apm state: add escape hatch from in_design back to new"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "36266"
branch = "ticket/8b3fc691-apm-state-add-escape-hatch-from-in-desig"
created_at = "2026-03-30T14:44:59.243807Z"
updated_at = "2026-03-30T18:08:40.819291Z"
+++

## Spec

### Problem

When a worker crashes or is killed while a ticket is `in_design` or `in_progress`,
the ticket is stuck. The state machine only allows forward transitions from those
states (e.g. `in_design → specd`, `in_progress → implemented`), so a supervisor
cannot reset the ticket without directly editing the branch blob.

### Acceptance criteria

- [x] `apm state <id> new --force` succeeds when the ticket is in `in_design`,
      bypassing the normal transition check
- [x] `apm state <id> ready --force` succeeds when the ticket is in `in_progress`,
      bypassing the normal transition check
- [x] `apm state <id> <state> --force` works from any non-terminal state to any
      valid state (the target state must still exist in the config)
- [x] Without `--force`, the normal transition rules continue to be enforced
- [x] `--force` does not bypass the document-level validations (`specd` must still
      have a valid spec; `implemented` must still have all criteria checked)
- [x] The history row is appended as normal
- [x] `apm state --help` mentions the `--force` flag

### Out of scope

- Role-based access control (no actor check — APM has no runtime auth)
- Adding `--force` to any command other than `apm state`
- Changes to `apm.toml` or the state machine definition

### Approach

1. Add `#[arg(long)] force: bool` to the `State` subcommand in `apm/src/main.rs`
2. Thread `force` through to `cmd::state::run()` (update its signature)
3. In `state.rs`, wrap the transition-enforcement block in `if !force { … }` —
   the block at lines 45-66 that bails when no matching transition is found
4. Document-level validations (the `match new_state.as_str()` block) are NOT
   skipped — they remain unconditional
5. No changes to `apm.toml`, config types, or any other command

### Open questions



### Amendment requests



### Code review
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:44Z | — | new | philippepascal |
| 2026-03-30T16:09Z | new | in_design | philippepascal |
| 2026-03-30T16:10Z | in_design | specd | claude-0330-1620-b7e2 |
| 2026-03-30T16:13Z | specd | ready | philippepascal |
| 2026-03-30T16:14Z | ready | in_progress | philippepascal |
| 2026-03-30T16:25Z | in_progress | implemented | claude-0330-1630-c4d1 |
| 2026-03-30T16:26Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |