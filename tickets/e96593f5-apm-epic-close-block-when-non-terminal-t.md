+++
id = "e96593f5"
title = "apm epic close: block when non-terminal tickets exist; add --close-all to cascade"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e96593f5-apm-epic-close-block-when-non-terminal-t"
created_at = "2026-05-31T03:26:36.317944Z"
updated_at = "2026-06-01T02:57:17.529086Z"
+++

## Spec

### Problem

Make apm epic close consistent with its tickets.

CURRENT BEHAVIOUR (apm/src/cmd/epic.rs::run_close lines 73-132):
- Quiescence check via epic_is_quiescent — fails when tickets are in active states with live workers
- If quiescent, proceeds to push the epic branch and open or update a PR
- If the epic branch is already merged into default, deletes the branch with no PR
- DOES NOT touch any of the epic's tickets

PROBLEM:
Tickets in the epic that are in non-terminal supervisor-owned states (e.g., implemented, specd, blocked) pass the quiescence check but remain in those states after the epic closes. The epic disappears as a managed unit; the orphaned tickets persist. The supervisor then needs to remember to run apm sync (which catches implemented + branch-merged) or close each ticket manually. Easy to forget; inconsistent semantic of close.

PROPOSED BEHAVIOUR:

1. apm epic close <id> (no flag):
   - Quiescence check stays.
   - NEW: if any ticket in the epic is in a non-terminal state, bail with a clear message:
     'epic has N non-terminal tickets:
        <id1>  <state>  <title>
        <id2>  <state>  <title>
        ...
      Re-run with --close-all to cascade close, or close them manually first.'
   - This explicitly forces the supervisor to choose between cascading or doing it by hand. No silent orphaning.

2. apm epic close <id> --close-all:
   - Quiescence check stays.
   - Iterate over every ticket in the epic. For each:
     - If terminal (closed): skip (already done)
     - If in implemented or another supervisor-clean state: transition to closed
     - If in blocked or question: BAIL — these represent unresolved decisions; cascade close would lose information. Same error format as above. The supervisor must resolve them manually first.
   - After all tickets are closed, perform the existing epic close steps (push, PR, branch delete).

3. Error / progress messages should be clear about what is happening:
   - 'closing ticket #abc12345 ... done'
   - 'closing ticket #def67890 ... failed (in blocked state; resolve manually before retrying)'

EXTENDED QUIESCENCE:
Today epic_is_quiescent in apm-core/src/epic.rs probably checks for active states (in_progress, in_design) and live workers. The --close-all path should extend this with the blocked/question check before attempting any close. Decide in epic.rs (one helper) rather than scattering the logic.

TESTS:
- apm epic close <id> on an epic with no tickets in non-terminal states: behaviour unchanged from today.
- apm epic close <id> on an epic with an implemented ticket: bails with the new message; epic is not closed.
- apm epic close <id> --close-all on an epic with an implemented ticket: ticket transitions to closed; epic closes.
- apm epic close <id> --close-all on an epic with a blocked ticket: bails with a clear message; neither the ticket nor the epic is touched.
- apm epic close <id> --close-all on an epic with mixed states (some implemented, some blocked): bails on the blocked ones; neither cascades nor closes the epic.
- apm epic close <id> --close-all on an epic with only implemented and closed tickets: cascades close on the implemented ones; closes the epic.

OUT OF SCOPE:
- apm refresh-epic cascade-into-ticket-branches. Separate concern, separate ticket.
- The refresh-epic --merge push bug. Separate ticket.
- Renaming or refactoring the existing epic subcommands (new, close, refresh-epic, etc.) beyond adding the flag.
- Changes to apm sync's ready-to-close detection. That remains the path for tickets already merged into main via squash or normal merge.

REFERENCES:
- apm/src/cmd/epic.rs::run_close (lines 73-132)
- apm-core/src/epic.rs::epic_is_quiescent
- apm-core/src/ticket.rs and apm-core/src/state.rs for the close transition mechanics

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
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:57Z | groomed | in_design | philippepascal |
