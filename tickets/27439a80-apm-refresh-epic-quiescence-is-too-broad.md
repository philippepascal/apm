+++
id = "27439a80"
title = "apm refresh-epic quiescence is too broad: it blocks on tickets with no real work yet"
state = "implemented"
priority = 7
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/27439a80-apm-refresh-epic-quiescence-is-too-broad"
created_at = "2026-05-29T22:07:24.444794Z"
updated_at = "2026-05-29T22:30:34.222953Z"
+++

## Spec

### Problem

BUG: epic_is_quiescent (apm-core/src/epic.rs:30) marks a ticket as 'not quiescent' when its state is non-terminal AND non-worker_end. By that rule a brand-new ticket — created with apm new but never started, no branch content beyond the ticket .md, no worktree, no scheduled work — blocks apm refresh-epic --merge / --pr / --auto. Observed example: 10 tickets in state 'new' on epic 72294403 caused refresh-epic --auto to bail with 'cannot refresh epic: the following tickets are not quiescent'. Those tickets cannot conflict with a main->epic merge because they have no implementation work to disturb. The current rule is conservative beyond its purpose.

ROOT CAUSE: the quiescence check uses ONLY the state-config flags (terminal, worker_end) as proxies for 'no work in flight.' But a ticket that has never entered implementation has no committed worktree code to conflict with regardless of which non-terminal, non-worker_end state it currently sits in. The narrow state-flag heuristic is too blunt.

FIX (direction; spec-writer to refine): align quiescence with the implementation-reached signal that ticket ada017c0 already established. ada017c0 added Config::implementation_state_ids() (transition fields, order-independent) plus ticket_fmt::history_target_states() and uses 'current state in impl_states OR history shows entry into impl_states' to decide whether a ticket has reached implementation. Reuse the same predicate here: a ticket should block quiescence ONLY if it has reached an implementation state (i.e. real code work may exist on its branch) AND is not in a terminal state. Tickets that have never entered implementation are quiescent regardless of their current state. Terminal tickets are quiescent regardless of their history — the work is done and merged.

The live-worker check (apm-core/src/worker.rs is_alive, applied inside epic_is_quiescent below the state-check) stays unchanged and independent — it catches any ticket with a running process regardless of state.

OUTCOME: with the above, a 'new' ticket no longer blocks. A 'groomed'/'specd' ticket without history of entering implementation no longer blocks. A 'ready' ticket without implementation history no longer blocks. An 'in_progress' / 'implemented' / 'merge_failed' ticket DOES block (it has reached impl and is not terminal). An 'ammend' ticket whose history shows it was previously in_progress DOES block. A terminal ('closed') ticket does NOT block even if its history contains an implementation state — the terminal exclusion takes priority.

Reviewer note: under today's rule, worker_end states (including 'implemented' in most workflows) are excluded from blocking. The new rule treats 'implemented' as a blocker because it is in implementation_state_ids() and is not a terminal state. This is a deliberate stricter change: an 'implemented' ticket may have unmerged branch content, and treating it as quiescent would be incorrect.

OUT OF SCOPE: changes to the refresh-epic command surface (inform/--merge/--pr/--auto modes from 12f2c7fa); the inform mode still skips quiescence entirely; the live-worker check; the sync close-eligibility logic (ada017c0) — only the quiescence predicate changes.

TESTS: existing epic_is_quiescent_* unit tests in epic.rs must still pass (state_blocker test currently uses a non-worker_end state — the spec-writer will need to update fixtures to ensure that test's ticket has entered implementation, or replace the fixture). Add unit tests: a 'new' ticket on an epic does NOT block; an 'implemented'-state ticket with history through in_progress DOES block; an 'ammend'-state ticket whose history shows it previously reached in_progress DOES block; a 'closed' (terminal) ticket with implementation history does NOT block; quiescence is invariant to [[workflow.states]] order (per ada017c0's invariance discipline).

### Acceptance criteria

- [x] `apm refresh-epic --auto` on an epic whose tickets are all in pre-implementation states (`new`, `groomed`, `specd`, `ready`) with no implementation history reports zero blockers
- [x] A ticket currently in an implementation state (e.g. `in_progress`) blocks quiescence
- [x] A ticket currently in a post-implementation state (e.g. `implemented`) blocks quiescence
- [x] A ticket in `ammend` whose History table shows a prior transition into an implementation state (e.g. `in_progress`) blocks quiescence
- [x] A closed (terminal) ticket whose History shows entry into an implementation state (e.g. `in_progress`) does NOT block quiescence
- [x] A ticket in `ready` with no implementation history does not block quiescence
- [x] The live-worker check still independently blocks a ticket whose worktree has a live `.apm-worker.pid`, regardless of state
- [x] `epic_is_quiescent` results are invariant to the ordering of `[[workflow.states]]` in `apm.toml`
- [x] All existing `cargo test --workspace` tests pass

### Out of scope

- Changes to the `refresh-epic` command surface (inform/--merge/--pr/--auto modes from 12f2c7fa)
- The `inform` mode, which already skips quiescence entirely
- The live-worker check in `apm-core/src/worker.rs`
- The sync close-eligibility logic from ticket ada017c0
- `Config::implementation_state_ids()` and `ticket_fmt::history_target_states()` — reused unchanged
- Behaviour when `implementation_state_ids()` returns an empty set (workflows with no implementation-flavoured transitions); in that case no state blocks, which is the correct conservative outcome

### Approach

#### Change to `epic_is_quiescent` (`apm-core/src/epic.rs`)

Hoist `config.implementation_state_ids()` and `config.terminal_state_ids()` before the ticket loop (both are constant for the call). Inside the loop, replace the current state-flag predicate with the implementation-reached check:

```rust
// Hoist before the loop:
let impl_states = config.implementation_state_ids();
let terminal_states = config.terminal_state_ids();

// Inside the loop, replace the state-blocks branch with:
let has_reached_impl = impl_states.contains(state_id.as_str())
    || crate::ticket_fmt::history_target_states(&t.body)
        .iter()
        .any(|s| impl_states.contains(s.as_str()));
if has_reached_impl && !terminal_states.contains(state_id.as_str()) {
    blockers.push(format!("  {id} — {title} (state: {state_id})"));
    continue;
}
```

Terminal tickets are excluded: a `closed` ticket that previously passed through `in_progress` has already been merged and should not block. The `is_worker_end` lookup becomes dead code; remove it. The `is_terminal` lookup is superseded by `terminal_states`; remove it too. The live-worker block (the `ticket_branch` / `worktrees` check that follows) is untouched.

The `has_reached_impl` predicate mirrors the one in `apm-core/src/sync.rs:29–33`. The terminal exclusion mirrors the gate applied in sync's Cases 1–4 (all subtract terminal states before acting).

#### Test updates (`apm-core/src/epic.rs` `#[cfg(test)]` block)

**Add `TOML_WITH_IMPL_STATES` constant** for the quiescence tests. This workflow has a coder `command:start` transition to `in_progress` and a `pr_or_epic_merge` completion to `implemented`, so `implementation_state_ids()` returns `{"in_progress", "implemented"}`. Include `ready`, `in_progress`, `implemented`, `ammend`, and `closed` states. Mark `closed` as `terminal = true`.

**Add `make_ticket_content_with_history` helper** that takes `(from, to)` row pairs and appends a `## History` table after the body, so `history_target_states` can find the `To` column values.

**Update `epic_is_quiescent_state_blocker`** — switch to `TOML_WITH_IMPL_STATES` and change the ticket state from `ready` to `in_progress` (an implementation state). Assert one blocker with `"(state: in_progress)"`.

**Add `epic_is_quiescent_ready_no_history_does_not_block`** — ticket in `ready` state, `TOML_WITH_IMPL_STATES`, no history. Assert `blockers.is_empty()`.

**Add `epic_is_quiescent_ammend_with_impl_history_blocks`** — ticket in `ammend` state with history rows `[("groomed", "in_progress"), ("in_progress", "ammend")]` via `make_ticket_content_with_history`. Assert one blocker.

**Add `epic_is_quiescent_closed_with_impl_history_does_not_block`** — ticket in `closed` state (terminal in `TOML_WITH_IMPL_STATES`) with history rows `[("in_progress", "implemented"), ("implemented", "closed")]` via `make_ticket_content_with_history`. Assert `blockers.is_empty()`.

**Add `epic_is_quiescent_order_invariant`** — build two configs from `TOML_WITH_IMPL_STATES` with `[[workflow.states]]` in reversed order; assert both produce the same blocker list for the same set of tickets.

The two existing tests `epic_is_quiescent_all_done` and `epic_is_quiescent_live_worker_blocker` use `TOML_WITH_WORKER_END` which has no impl transitions, so `impl_states` is empty and `terminal_states` covers whatever states are marked terminal. For `all_done`: `has_reached_impl` is false for both tickets → no blockers → still passes. For `live_worker_blocker`: the state check yields no blocker, but the live-worker path (unchanged) still fires → still passes.

### Open questions


### Amendment requests

- [x] Preserve the terminal-state exclusion in the new quiescence predicate. The Approach as written removes the is_terminal lookup and applies has_reached_impl unconditionally; this would cause a closed (terminal) ticket whose History contains an in_progress row — i.e. any normally-implemented-then-closed ticket on the epic — to be flagged as a blocker, which is wrong (the work is done and merged; nothing to disturb). Fix: hoist let terminal = config.terminal_state_ids() before the loop and require !terminal.contains(state_id) before pushing the blocker. Equivalently, filter terminal tickets out of the iterator. The shape mirrors the post-ada017c0 sync gates exactly (sync.rs Cases 1 to 4 + hint all subtract terminal). Also add a matching Acceptance Criterion: a closed (terminal) ticket whose History shows entry into an implementation state does NOT block quiescence — and a corresponding unit test (e.g. epic_is_quiescent_closed_with_impl_history_does_not_block) so the regression cannot slip through. Finally, in the Problem section's OUTCOME paragraph or in Out of scope, explicitly note that implemented tickets DO block under the new rule (because implemented is in implementation_state_ids), which is a deliberate stricter change from today's worker_end-excludes-implemented behavior — flag it so a code reviewer is not surprised.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T22:07Z | — | new | philippepascal |
| 2026-05-29T22:08Z | new | groomed | philippepascal |
| 2026-05-29T22:08Z | groomed | in_design | philippepascal |
| 2026-05-29T22:11Z | in_design | specd | claude |
| 2026-05-29T22:15Z | specd | ammend | philippepascal |
| 2026-05-29T22:16Z | ammend | in_design | philippepascal |
| 2026-05-29T22:20Z | in_design | specd | claude |
| 2026-05-29T22:24Z | specd | ready | philippepascal |
| 2026-05-29T22:24Z | ready | in_progress | philippepascal |
| 2026-05-29T22:30Z | in_progress | implemented | claude |
