+++
id = "27439a80"
title = "apm refresh-epic quiescence is too broad: it blocks on tickets with no real work yet"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/27439a80-apm-refresh-epic-quiescence-is-too-broad"
created_at = "2026-05-29T22:07:24.444794Z"
updated_at = "2026-05-29T22:08:12.815894Z"
+++

## Spec

### Problem

BUG: epic_is_quiescent (apm-core/src/epic.rs:30) marks a ticket as 'not quiescent' when its state is non-terminal AND non-worker_end. By that rule a brand-new ticket — created with apm new but never started, no branch content beyond the ticket .md, no worktree, no scheduled work — blocks apm refresh-epic --merge / --pr / --auto. Observed example: 10 tickets in state 'new' on epic 72294403 caused refresh-epic --auto to bail with 'cannot refresh epic: the following tickets are not quiescent'. Those tickets cannot conflict with a main->epic merge because they have no implementation work to disturb. The current rule is conservative beyond its purpose.

ROOT CAUSE: the quiescence check uses ONLY the state-config flags (terminal, worker_end) as proxies for 'no work in flight.' But a ticket that has never entered implementation has no committed worktree code to conflict with regardless of which non-terminal, non-worker_end state it currently sits in. The narrow state-flag heuristic is too blunt.

FIX (direction; spec-writer to refine): align quiescence with the implementation-reached signal that ticket ada017c0 already established. ada017c0 added Config::implementation_state_ids() (transition fields, order-independent) plus ticket_fmt::history_target_states() and uses 'current state in impl_states OR history shows entry into impl_states' to decide whether a ticket has reached implementation. Reuse the same predicate here: a ticket should block quiescence ONLY if it has reached an implementation state (i.e. real code work may exist on its branch). Tickets that have never entered implementation are quiescent regardless of their current state.

The live-worker check (apm-core/src/worker.rs is_alive, applied inside epic_is_quiescent below the state-check) stays unchanged and independent — it catches any ticket with a running process regardless of state.

OUTCOME: with the above, a 'new' ticket no longer blocks. A 'groomed'/'specd' ticket without history of entering implementation no longer blocks. A 'ready' ticket without implementation history no longer blocks. An 'in_progress' / 'implemented' / 'merge_failed' ticket DOES block (it has reached impl). An 'ammend' ticket whose history shows it was previously in_progress DOES block.

OUT OF SCOPE: changes to the refresh-epic command surface (inform/--merge/--pr/--auto modes from 12f2c7fa); the inform mode still skips quiescence entirely; the live-worker check; the sync close-eligibility logic (ada017c0) — only the quiescence predicate changes.

TESTS: existing epic_is_quiescent_* unit tests in epic.rs must still pass (state_blocker test currently uses a non-worker_end state — the spec-writer will need to update fixtures to ensure that test's ticket has entered implementation, or replace the fixture). Add unit tests: a 'new' ticket on an epic does NOT block; an 'implemented'-state ticket with history through in_progress DOES block; an 'ammend'-state ticket whose history shows it previously reached in_progress DOES block; quiescence is invariant to [[workflow.states]] order (per ada017c0's invariance discipline).

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
| 2026-05-29T22:07Z | — | new | philippepascal |
| 2026-05-29T22:08Z | new | groomed | philippepascal |
