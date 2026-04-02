+++
id = "1cadf37f"
title = "UI: show dep-blocked status on tickets waiting in queue"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/1cadf37f-ui-show-dep-blocked-status-on-tickets-wa"
created_at = "2026-04-02T23:21:21.478449Z"
updated_at = "2026-04-02T23:23:31.239952Z"
+++

## Spec

### Problem

A ticket in `groomed` (or any actionable state) that has unsatisfied `depends_on` deps is silently invisible in the work queue — `apm next` skips it and no worker picks it up. The supervisor board shows the ticket in its swimlane as if it is ready for dispatch, with no visual indication that it is actually dep-blocked.

This creates confusion: the supervisor sees a `groomed` ticket sitting in the column, assumes it will be picked up shortly, and has no immediate way to know it is waiting on another ticket that hasn't reached the required dep gate yet. Diagnosing the stall requires manually running `apm show` and cross-referencing dep states.

The fix is to make dep-blocked status visible directly on the ticket card in the supervisor board. The server already computes `blocking_deps` in ticket responses (used for the existing lock icon on `ready` tickets); the same field should drive a clear visual on any ticket whose actionable state is gated by a dep that hasn't cleared. The card should show which tickets are blocking it and their current states, so the supervisor can act without leaving the board.

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
| 2026-04-02T23:21Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
