+++
id = "94a38059"
title = "Supervisor board should show tickets in new state"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "81824"
branch = "ticket/94a38059-supervisor-board-should-show-tickets-in-"
created_at = "2026-04-02T03:17:21.639407Z"
updated_at = "2026-04-02T19:07:24.645454Z"
+++

## Spec

### Problem

The supervisor board (apm-ui) hard-codes which ticket states appear as swimlane columns in the `SUPERVISOR_STATES` constant inside `apm-ui/src/lib/supervisorUtils.ts`. The `new` state is not in that list, so newly-created tickets are invisible on the board until a supervisor manually transitions them to `groomed`.

Every ticket is born in `new` state (`apm-core/src/ticket.rs`). Supervisors are supposed to review `new` tickets and promote them to `groomed` (or close them), but they cannot see those tickets on the board — the primary tool for day-to-day supervision. They must resort to `apm list --state new` on the CLI, which breaks the board-centric workflow.

Adding `new` to the visible states lets supervisors act on newly-created tickets directly from the board.

### Acceptance criteria

- [x] The supervisor board renders a swimlane column for the `new` state
- [x] Tickets in `new` state appear as cards inside that column
- [x] The `new` column is visible by default (no extra toggle required)
- [x] The `new` column displays the correct state label ("new")
- [x] Existing columns for all other states (question, specd, ammend, blocked, implemented, accepted) are unaffected

### Out of scope

- Adding `groomed`, `in_design`, `ready`, `in_progress`, or `closed` to the default board view
- Any changes to the state machine or allowed transitions
- Column ordering or layout changes beyond inserting the new column
- A toggle/filter UI for showing/hiding individual states
- Backend or CLI changes

### Approach

**File: `apm-ui/src/lib/supervisorUtils.ts`**

Prepend `'new'` to the `SUPERVISOR_STATES` array. Current value:

```ts
export const SUPERVISOR_STATES = ['question', 'specd', 'ammend', 'blocked', 'implemented', 'accepted']
```

New value:

```ts
export const SUPERVISOR_STATES = ['new', 'question', 'specd', 'ammend', 'blocked', 'implemented', 'accepted']
```

Placing `new` first puts it at the leftmost column, matching the natural left-to-right flow of the ticket lifecycle.

**File: `apm-ui/src/lib/stateColors.ts`** (conditional)

Inspect the file. If `new` already has a colour entry, no change needed. If it falls through to a default, add an explicit entry using the grey colour that matches `workflow.toml` (e.g. `new: GRAY`).

No other files need changing. `SupervisorView.tsx`, `Swimlane.tsx`, and `TicketCard.tsx` are all state-agnostic and pick up the new state automatically.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T03:17Z | — | new | apm |
| 2026-04-02T16:56Z | new | groomed | apm |
| 2026-04-02T16:58Z | groomed | in_design | philippepascal |
| 2026-04-02T17:00Z | in_design | specd | claude-0402-1700-s4w1 |
| 2026-04-02T17:23Z | specd | ready | apm |
| 2026-04-02T17:25Z | ready | in_progress | philippepascal |
| 2026-04-02T17:27Z | in_progress | implemented | claude-0402-1725-w9k2 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |