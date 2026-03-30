+++
id = "9d56726c"
title = "refactor: thin out medium cmd files (list, set, take, workers, worktrees, work)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "philippepascal"
branch = "ticket/9d56726c-refactor-thin-out-medium-cmd-files-list-"
created_at = "2026-03-30T14:27:53.108961Z"
updated_at = "2026-03-30T16:36:13.835668Z"
+++

## Spec

### Problem

Several medium-sized CLI command files contain filtering, mutation, or process
monitoring logic that should live in `apm-core`:

**list.rs (40 lines):** Ticket filtering logic (terminal state, actionable state,
agent/supervisor filters) duplicated from other commands. Should call a shared
`apm_core::ticket::list_filtered()`.

**set.rs (65 lines):** Field validation (priority/effort/risk range checks) and
immutability enforcement (author field) belong in `apm-core::ticket::set_field()`.

**take.rs (78 lines):** Agent handoff validation and history append belong in
`apm_core::ticket::handoff()`.

**workers.rs (277 lines):** PID file parsing, process liveness checks (`kill -0`),
elapsed time calculation, and kill logic are business logic. Only the table
formatting belongs in the CLI.

**worktrees.rs (67 lines):** Worktree enumeration with branch-to-ticket matching
should delegate to `apm_core::git::list_worktrees_with_tickets()`.

**work.rs (93 lines):** Worker pool management and result state validation should
delegate more cleanly to `apm-core` rather than calling across cmd modules.

Individually small, but collectively ~600 lines of leaked business logic that
blocks `apm-serve` from reusing any of it.

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
| 2026-03-30T16:36Z | new | in_design | philippepascal |
