+++
id = "27439a80"
title = "apm refresh-epic quiescence is too broad: it blocks on tickets with no real work yet"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/27439a80-apm-refresh-epic-quiescence-is-too-broad"
created_at = "2026-05-29T22:07:24.444794Z"
updated_at = "2026-05-29T22:11:27.935920Z"
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

- [ ] `apm refresh-epic --auto` on an epic whose tickets are all in pre-implementation states (`new`, `groomed`, `specd`, `ready`) with no implementation history reports zero blockers
- [ ] A ticket currently in an implementation state (e.g. `in_progress`) blocks quiescence
- [ ] A ticket currently in a post-implementation state (e.g. `implemented`) blocks quiescence
- [ ] A ticket in `ammend` whose History table shows a prior transition into an implementation state (e.g. `in_progress`) blocks quiescence
- [ ] A ticket in `ready` with no implementation history does not block quiescence
- [ ] The live-worker check still independently blocks a ticket whose worktree has a live `.apm-worker.pid`, regardless of state
- [ ] `epic_is_quiescent` results are invariant to the ordering of `[[workflow.states]]` in `apm.toml`
- [ ] All existing `cargo test --workspace` tests pass

### Out of scope

- Changes to the `refresh-epic` command surface (inform/--merge/--pr/--auto modes from 12f2c7fa)
- The `inform` mode, which already skips quiescence entirely
- The live-worker check in `apm-core/src/worker.rs`
- The sync close-eligibility logic from ticket ada017c0
- `Config::implementation_state_ids()` and `ticket_fmt::history_target_states()` — reused unchanged
- Behaviour when `implementation_state_ids()` returns an empty set (workflows with no implementation-flavoured transitions); in that case no state blocks, which is the correct conservative outcome

### Approach

#### Change to `epic_is_quiescent` (`apm-core/src/epic.rs`)

Hoist `config.implementation_state_ids()` before the ticket loop (it is constant for the call). Inside the loop, replace the current state-flag predicate with the implementation-reached check:

```rust
// Remove is_terminal / is_worker_end / state_blocks; hoist before the loop:
let impl_states = config.implementation_state_ids();

// Inside the loop, replace the state-blocks branch with:
let has_reached_impl = impl_states.contains(state_id.as_str())
    || crate::ticket_fmt::history_target_states(&t.body)
        .iter()
        .any(|s| impl_states.contains(s.as_str()));
if has_reached_impl {
    blockers.push(format!("  {id} — {title} (state: {state_id})"));
    continue;
}
```

The `state_cfg`, `is_terminal`, and `is_worker_end` lookups become dead code; remove them. The live-worker block (the `ticket_branch` / `worktrees` check that follows) is untouched.

The `has_reached_impl` predicate mirrors the one in `apm-core/src/sync.rs:29–33` used for close-eligibility detection.

#### Test updates (`apm-core/src/epic.rs` `#[cfg(test)]` block)

**Add `TOML_WITH_IMPL_STATES` constant** for the quiescence tests. This workflow has a coder `command:start` transition to `in_progress` and a `pr_or_epic_merge` completion to `implemented`, so `implementation_state_ids()` returns `{"in_progress", "implemented"}`. Include `ready`, `in_progress`, `implemented`, `ammend`, and `closed` states.

**Add `make_ticket_content_with_history` helper** that takes `(from, to)` row pairs and appends a `## History` table after the body, so `history_target_states` can find the `To` column values.

**Update `epic_is_quiescent_state_blocker`** — switch to `TOML_WITH_IMPL_STATES` and change the ticket state from `ready` to `in_progress` (an implementation state). Assert one blocker with `"(state: in_progress)"`.

**Add `epic_is_quiescent_ready_no_history_does_not_block`** — ticket in `ready` state, `TOML_WITH_IMPL_STATES`, no history. Assert `blockers.is_empty()`.

**Add `epic_is_quiescent_ammend_with_impl_history_blocks`** — ticket in `ammend` state with history rows `[("groomed", "in_progress"), ("in_progress", "ammend")]` via `make_ticket_content_with_history`. Assert one blocker.

**Add `epic_is_quiescent_order_invariant`** — build two configs from `TOML_WITH_IMPL_STATES` with `[[workflow.states]]` in reversed order; assert both produce the same blocker list for the same set of tickets.

The two existing tests `epic_is_quiescent_all_done` and `epic_is_quiescent_live_worker_blocker` use `TOML_WITH_WORKER_END` which has no impl transitions, so `impl_states` is empty. For `all_done`: `has_reached_impl` is false for both tickets → no blockers → still passes. For `live_worker_blocker`: the state check yields no blocker, but the live-worker path (unchanged) still fires → still passes.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T22:07Z | — | new | philippepascal |
| 2026-05-29T22:08Z | new | groomed | philippepascal |
| 2026-05-29T22:08Z | groomed | in_design | philippepascal |