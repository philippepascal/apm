+++
id = "4258e031"
title = "Priority should reflect critical path through depends_on graph"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "apm"
agent = "86601"
branch = "ticket/4258e031-priority-should-reflect-critical-path-th"
created_at = "2026-04-02T05:23:36.689810Z"
updated_at = "2026-04-02T19:06:34.433435Z"
+++

## Spec

### Problem

Once `depends_on` is in use, the raw priority score of a blocking ticket no longer reflects its true urgency. A root dependency with low priority sits near the bottom of the queue and of `apm next` scoring, even if it is blocking a chain of high-value work. There is no visual or scheduling signal that dispatching it first matters.

The correct effective priority of any ticket is `max(own_priority, max priority of all direct and transitive dependents)`. For example: if A (priority 2) blocks B (priority 9), A's effective priority should be 9 — dispatching A first is what unlocks B.

Without critical-path elevation, the priority queue and `apm next` give a misleading picture once dependency graphs are in use. Agents and supervisors have to manually reason about the graph instead of trusting the queue order.

### Acceptance criteria

- [x] `apm next` returns a lower-raw-priority ticket X before a higher-raw-priority ticket Y when X is a direct or transitive dependency of a ticket whose raw priority exceeds Y's raw priority
- [x] The `/api/queue` response lists a blocking ticket above independent tickets with higher raw priority when the blocker's effective priority (from its dependents) is higher
- [x] A ticket with no dependents sorts by its own raw priority, unchanged from current behavior
- [x] Effective priority propagates transitively: if A (priority 2) blocks B (priority 5) which blocks C (priority 9), A's effective priority is 9
- [x] A dependency cycle (A depends on B, B depends on A) does not panic or loop infinitely
- [x] The `priority` field stored in ticket TOML frontmatter is not modified by `apm next` or queue queries
- [x] Each entry in the `/api/queue` response includes an `effective_priority` field (u8) reflecting the elevated value

### Out of scope

- Modifying the stored `priority` field in ticket frontmatter
- Displaying in the UI or CLI which dependent(s) caused a ticket's priority to be elevated. The UI/CLI should eventually surface this — e.g. "effective priority 9 (driven by #abc123)" — but it is deferred. The `effective_priority` field on `QueueEntry` is the natural anchor; a companion `priority_driver_id` field could be added in a follow-on ticket without a schema change.
- Cycle detection beyond a simple visited-set (no topological-sort guarantee required)
- Changing `apm set <id> priority` behaviour
- Priority elevation for tickets already in terminal or satisfies_deps states (they are filtered out of the actionable list)

### Approach

At query time, build a reverse dependency index from the active (non-terminal, non-satisfies_deps) tickets and propagate max priority up the graph. No stored fields are mutated.

**Files that change**

`apm-core/src/ticket.rs`:

1. Add `pub fn build_reverse_index<'a>(tickets: &'a [Ticket]) -> HashMap<&'a str, Vec<&'a Ticket>>`: iterate the provided tickets; for each ID in `depends_on`, push the current ticket into `map[dep_id]`. Tickets without `depends_on` contribute nothing. The caller is responsible for passing only non-terminal, non-satisfies_deps tickets — closed tickets that once depended on X must not inflate X's effective priority after the work is done.

2. Add `pub fn effective_priority(ticket: &Ticket, reverse_index: &HashMap<&str, Vec<&Ticket>>) -> u8`: BFS from `ticket.frontmatter.id` over the reverse index using a `HashSet<&str>` visited set. Collect `frontmatter.priority` of every reachable dependent; return `max(ticket.frontmatter.priority, max_dependent_priority)`.

3. Modify `sorted_actionable`: call `build_reverse_index(actionable_tickets)` once before sorting, passing the same filtered slice used for actionability. In the sort closure, replace each `t.score(pw, ew, rw)` call with an inline formula using `effective_priority(t, &rev_idx)` in place of `t.frontmatter.priority`. The existing `score()` method is unchanged; only the sort closure diverges.

`apm-server/src/queue.rs`:

4. Add `effective_priority: u8` field to `QueueEntry`.

5. In `queue_handler`, filter `tickets` to the non-terminal, non-satisfies_deps set first. Build the reverse index once from that filtered set before the `.map()` loop. Set `effective_priority` from `effective_priority(t, &rev_idx)` on each entry; compute `score` using the same elevated value so it stays consistent with the sort order returned by `sorted_actionable`.

**Performance note**

`apm next` calls `sorted_actionable` on every invocation, so `build_reverse_index` is called once per invocation — acceptable at ticket scale. For `apm list` and the `/api/queue` endpoint, which may be called frequently, the reverse index and all derived effective priorities must be built **once per request** and reused across the response. Do not call `build_reverse_index` inside a per-ticket loop (e.g. inside `.map()`) — that would make it O(n²) unnecessarily.

Filtering to non-terminal, non-satisfies_deps tickets before building the index is also important for long-term correctness: as a project accumulates closed tickets, passing them all to `build_reverse_index` would let finished work inflate the effective priority of unrelated open tickets. The filtering cost is O(n) and stays negligible.

**Step order**

1. Implement `build_reverse_index` and `effective_priority` in `ticket.rs` with unit tests covering: single-hop elevation, transitive elevation, no-dependents (identity), cycle safety, and a closed-dependent that must not elevate the blocker.
2. Modify `sorted_actionable` to use effective priority in the sort closure; add unit tests for the new ordering.
3. Update `QueueEntry` and the handler in `queue.rs`.
4. Add an integration test verifying that `pick_next` selects the low-priority blocking ticket first when its dependent has higher priority.
5. Run `cargo test --workspace` — all tests must pass.

### Open questions


### Amendment requests

- [x] Fix AC #4 direction: "A (priority 2) is blocked by B (priority 5) which is blocked by C (priority 9)" should read "A (priority 2) blocks B (priority 5) which blocks C (priority 9), A's effective priority is 9". A is the prerequisite that must ship first; the current wording reverses the chain and makes A the last step with no reason to be elevated.
- [x] Add a performance consideration to the Approach: `apm next` calls `sorted_actionable` on every invocation so the reverse index is built once per call — acceptable at ticket scale. However `apm list` and the `/api/queue` endpoint may be called frequently; for those code paths the reverse index (and effective priorities derived from it) should be built once per request and reused across the response, not recomputed per ticket. Note this explicitly so the implementer does not accidentally call `build_reverse_index` inside a per-ticket loop.
- [x] Add a visual consideration to Out of scope or a new ### Visual considerations section: the UI and/or CLI should eventually surface *which* ticket in the dependency graph is responsible for elevating a ticket's position — e.g. "effective priority 9 (driven by #abc123)". This is explicitly out of scope for the current ticket but should be called out so it is not forgotten; the `effective_priority` field on `QueueEntry` should carry enough information (or a companion `priority_driver_id` field) to make this possible later.
- [x] Exclude terminal and satisfies_deps tickets from `build_reverse_index`: the Approach currently says to build the index from "all loaded tickets". A closed ticket that depends on X would incorrectly inflate X's effective priority even though that dependent is already done and X is already unblocked. The index must be built from the same non-terminal, non-satisfies_deps set used for actionability filtering. Also update the performance note to reflect that the growing body of closed tickets is the main reason this filter matters long-term.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:23Z | — | new | apm |
| 2026-04-02T16:57Z | new | groomed | apm |
| 2026-04-02T17:01Z | groomed | in_design | philippepascal |
| 2026-04-02T17:05Z | in_design | specd | claude-0402-1701-b7f2 |
| 2026-04-02T17:39Z | specd | ammend | apm |
| 2026-04-02T17:59Z | ammend | in_design | philippepascal |
| 2026-04-02T18:01Z | in_design | specd | claude-0402-1759-e6e0 |
| 2026-04-02T18:09Z | specd | ammend | apm |
| 2026-04-02T18:10Z | ammend | in_design | philippepascal |
| 2026-04-02T18:12Z | in_design | specd | claude-0402-1810-c9d1 |
| 2026-04-02T18:14Z | specd | ready | apm |
| 2026-04-02T18:16Z | ready | in_progress | philippepascal |
| 2026-04-02T18:23Z | in_progress | implemented | claude-0402-1816-x9k2 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |