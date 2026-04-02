+++
id = "4258e031"
title = "Priority should reflect critical path through depends_on graph"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "apm"
agent = "11220"
branch = "ticket/4258e031-priority-should-reflect-critical-path-th"
created_at = "2026-04-02T05:23:36.689810Z"
updated_at = "2026-04-02T17:05:10.485429Z"
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

- Modifying the stored `priority` field in ticket frontmatter
- Displaying in the UI which dependent(s) caused a ticket's priority to be elevated
- Cycle detection beyond a simple visited-set (no topological-sort guarantee required)
- Changing `apm set <id> priority` behaviour
- Priority elevation for tickets already in terminal or satisfies_deps states (they are filtered out of the actionable list)

### Approach

At query time, build a reverse dependency index from all loaded tickets and propagate max priority up the graph. No stored fields are mutated.

**Files that change**

`apm-core/src/ticket.rs`:

1. Add `pub fn build_reverse_index<'a>(tickets: &'a [Ticket]) -> HashMap<&'a str, Vec<&'a Ticket>>`: iterate all tickets; for each ID in `depends_on`, push the current ticket into `map[dep_id]`. Tickets without `depends_on` contribute nothing.

2. Add `pub fn effective_priority(ticket: &Ticket, reverse_index: &HashMap<&str, Vec<&Ticket>>) -> u8`: BFS from `ticket.frontmatter.id` over the reverse index using a `HashSet<&str>` visited set. Collect `frontmatter.priority` of every reachable dependent; return `max(ticket.frontmatter.priority, max_dependent_priority)`.

3. Modify `sorted_actionable`: call `build_reverse_index(tickets)` once before sorting. In the sort closure, replace each `t.score(pw, ew, rw)` call with an inline formula using `effective_priority(t, &rev_idx)` in place of `t.frontmatter.priority`. The existing `score()` method is unchanged; only the sort closure diverges.

`apm-server/src/queue.rs`:

4. Add `effective_priority: u8` field to `QueueEntry`.

5. In `queue_handler`, build the reverse index once (from `tickets`) before the `.map()` loop. Set `effective_priority` from `effective_priority(t, &rev_idx)` on each entry; compute `score` using the same elevated value so it stays consistent with the sort order returned by `sorted_actionable`.

**Step order**

1. Implement `build_reverse_index` and `effective_priority` in `ticket.rs` with unit tests covering: single-hop elevation, transitive elevation, no-dependents (identity), and cycle safety.
2. Modify `sorted_actionable` to use effective priority in the sort closure; add unit tests for the new ordering.
3. Update `QueueEntry` and the handler in `queue.rs`.
4. Add an integration test verifying that `pick_next` selects the low-priority blocking ticket first when its dependent has higher priority.
5. Run `cargo test --workspace` — all tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:23Z | — | new | apm |
| 2026-04-02T16:57Z | new | groomed | apm |
| 2026-04-02T17:01Z | groomed | in_design | philippepascal |