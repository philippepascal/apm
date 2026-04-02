+++
id = "94a38059"
title = "Supervisor board should show tickets in new state"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "50942"
branch = "ticket/94a38059-supervisor-board-should-show-tickets-in-"
created_at = "2026-04-02T03:17:21.639407Z"
updated_at = "2026-04-02T16:58:11.384007Z"
+++

## Spec

### Problem

The supervisor board (apm-ui) hard-codes which ticket states appear as swimlane columns in the `SUPERVISOR_STATES` constant inside `apm-ui/src/lib/supervisorUtils.ts`. The `new` state is not in that list, so newly-created tickets are invisible on the board until a supervisor manually transitions them to `groomed`.

Every ticket is born in `new` state (`apm-core/src/ticket.rs`). Supervisors are supposed to review `new` tickets and promote them to `groomed` (or close them), but they cannot see those tickets on the board — the primary tool for day-to-day supervision. They must resort to `apm list --state new` on the CLI, which breaks the board-centric workflow.

Adding `new` to the visible states lets supervisors act on newly-created tickets directly from the board.

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
| 2026-04-02T03:17Z | — | new | apm |
| 2026-04-02T16:56Z | new | groomed | apm |
| 2026-04-02T16:58Z | groomed | in_design | philippepascal |