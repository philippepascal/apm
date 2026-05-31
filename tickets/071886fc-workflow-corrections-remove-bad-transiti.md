+++
id = "071886fc"
title = "Workflow corrections: remove bad transitions, restructure ammend path"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/071886fc-workflow-corrections-remove-bad-transiti"
created_at = "2026-05-31T02:57:20.412089Z"
updated_at = "2026-05-31T07:20:49.277920Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463"]
+++

## Spec

### Problem

The default `workflow.toml` (and the project's `.apm/workflow.toml`) contain three transitions that contradict the intended design:

**`in_design → ammend`**: A spec-writer encountering a blocker during design should route to `question`, not `ammend`. The `ammend` state is supervisor-initiated from `specd` or `implemented`; a spec-writer agent cannot self-request an amendment.

**`merge_failed → in_progress`**: When a merge fails, the correct recovery is to retry the merge (`merge_failed → implemented`) or escalate to the supervisor. Routing back to `in_progress` re-spawns the coder without new guidance and bypasses any supervisor review of the failure.

**`ammend → in_design` via `command:start`**: This creates two `command:start`-triggered paths into `in_design` (one from `groomed`, one from `ammend`), violating the trigger-uniqueness invariant planned for enforcement in the next ticket. The correct path is `ammend → groomed` (supervisor, manual), then `groomed → in_design` (command:start via `apm start`). The `ammend` state should become supervisor-actionable, since the only agent-dispatch path from it is being removed.

### Acceptance criteria

- [ ] `apm state <id> ammend` on a ticket in `in_design` returns an error (transition not defined)
- [ ] `apm state <id> in_progress` on a ticket in `merge_failed` returns an error (transition not defined)
- [ ] `apm state <id> groomed` on a ticket in `ammend` succeeds and the ticket reaches `groomed`
- [ ] `apm start` does not pick up tickets in `ammend` state (no `command:start` exits from `ammend`)
- [ ] A ticket can traverse `specd → ammend → groomed → in_design → specd` without error
- [ ] `apm list --actionable` does not include `ammend` tickets in the agent-actionable set
- [ ] The spec-writer agent prompts no longer instruct the agent to run `apm state <id> in_design` from `ammend`
- [ ] `cargo test --workspace` passes after all changes

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
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:20Z | groomed | in_design | philippepascal |