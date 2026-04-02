+++
id = "4258e031"
title = "Priority should reflect critical path through depends_on graph"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "11220"
branch = "ticket/4258e031-priority-should-reflect-critical-path-th"
created_at = "2026-04-02T05:23:36.689810Z"
updated_at = "2026-04-02T17:01:08.034857Z"
+++

## Spec

### Problem

Once `depends_on` is in use, the raw priority score of a blocking ticket no longer reflects its true urgency. A root dependency with low priority sits near the bottom of the queue and of `apm next` scoring, even if it is blocking a chain of high-value work. There is no visual or scheduling signal that dispatching it first matters.

The correct effective priority of any ticket is `max(own_priority, max priority of all direct and transitive dependents)`. For example: if A (priority 2) blocks B (priority 9), A's effective priority should be 9 — dispatching A first is what unlocks B.

Without critical-path elevation, the priority queue and `apm next` give a misleading picture once dependency graphs are in use. Agents and supervisors have to manually reason about the graph instead of trusting the queue order.

### Acceptance criteria

- [ ] `apm next` returns a lower-raw-priority ticket X before a higher-raw-priority ticket Y when X is a direct or transitive dependency of a ticket whose raw priority exceeds Y's raw priority
- [ ] The `/api/queue` response lists a blocking ticket above independent tickets with higher raw priority when the blocker's effective priority (from its dependents) is higher
- [ ] A ticket with no dependents sorts by its own raw priority, unchanged from current behavior
- [ ] Effective priority propagates transitively: if A (priority 2) is blocked by B (priority 5) which is blocked by C (priority 9), A's effective priority is 9
- [ ] A dependency cycle (A depends on B, B depends on A) does not panic or loop infinitely
- [ ] The `priority` field stored in ticket TOML frontmatter is not modified by `apm next` or queue queries
- [ ] Each entry in the `/api/queue` response includes an `effective_priority` field (u8) reflecting the elevated value

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

At query time, build a reverse dependency index from all loaded tickets and propagate max priority up the graph.

**1. Reverse dep index** — `HashMap<&str, Vec<&Ticket>>`: for each ticket with `depends_on`, add an entry in the map from each dep ID to the dependent ticket.

**2. Effective priority** — for each ticket, walk all direct and transitive dependents (BFS/DFS on the reverse index), collect their raw priority scores, and return `max(own_priority, max_dependent_priority)`. Cycles are safe to ignore (visit-set).

**3. `sorted_actionable`** — replace `t.frontmatter.priority` in the score formula with `effective_priority(t, &reverse_index)`.

**4. UI** — `TicketResponse` (or a new computed field `effective_priority`) carries the elevated score so the queue panel sorts correctly. The raw `priority` field stays unchanged.

**Out of scope**: modifying the stored `priority` field; UI display of why a ticket's priority was elevated; multi-level cycle detection beyond simple visit-set.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:23Z | — | new | apm |
| 2026-04-02T16:57Z | new | groomed | apm |
| 2026-04-02T17:01Z | groomed | in_design | philippepascal |