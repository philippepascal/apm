+++
id = "31429c7d"
title = "refactor: move state machine logic from state.rs and start.rs into apm-core"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
branch = "ticket/31429c7d-refactor-move-state-machine-logic-from-s"
created_at = "2026-03-30T14:27:29.706701Z"
updated_at = "2026-03-30T14:27:29.706701Z"
+++

## Spec

### Problem

`state.rs` and `start.rs` together contain ~780 lines of business logic that
belongs in `apm-core`:

**state.rs (274 lines):**
- State transition validation against config (allowed transitions, actor checks)
- Document validation before transition (spec sections required for "specd",
  AC checked for "implemented", amendment boxes for "ammend")
- History entry appending (`append_history`)
- Amendment section auto-creation
- PR creation via `gh` CLI (completion strategy logic)
- Merge into default branch with conflict handling
- Worktree provisioning for `in_design` state

**start.rs (509 lines):**
- Startable state detection from config
- State machine transition execution
- Worktree provisioning and merge-from-default
- Worker system prompt loading (`.apm/worker.md` fallback)
- PID file writing and cleanup thread
- Focus section extraction and clearing

These two files are tightly coupled — `start.rs` calls `state.rs` functions and
both manipulate the same state machine. Neither belongs in the CLI layer. `apm-serve`
will need to perform state transitions and start workers without shelling out to
the CLI or duplicating logic.

Target: `apm_core::state::transition()` and `apm_core::start::run()` with thin
CLI wrappers of ~30 lines each.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
