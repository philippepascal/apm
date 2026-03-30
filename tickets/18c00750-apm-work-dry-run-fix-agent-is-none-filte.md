+++
id = "18c00750"
title = "apm work --dry-run: fix agent.is_none() filter and use pick_next"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/18c00750-apm-work-dry-run-fix-agent-is-none-filte"
created_at = "2026-03-30T06:11:15.954147Z"
updated_at = "2026-03-30T06:17:05.071419Z"
+++

## Spec

### Problem

`run_dry` in `apm/src/cmd/work.rs` has two issues:

1. It still has the `fm.agent.is_none()` filter that was removed from `next.rs`
   and `start.rs` — causing it to skip `ready` tickets whose `agent` field was
   set by spec authorship, and report fewer candidates than will actually be
   dispatched.

2. It duplicates the candidate-filtering and sorting logic instead of calling
   `ticket::pick_next`, which was extracted specifically to avoid this drift.

### Acceptance criteria

- [ ] `apm work --dry-run` reports a `ready` ticket whose `agent` field is already set (e.g. by spec authorship) as a candidate — it is not silently skipped
- [ ] `apm work --dry-run` and `apm next` agree on which ticket would be dispatched first
- [ ] `run_dry` in `apm/src/cmd/work.rs` contains no inline filter-and-sort loop — it delegates to `ticket::pick_next`
- [ ] `apm work --dry-run` with no actionable tickets prints "dry-run: no actionable tickets" and exits 0
- [ ] `apm work --dry-run` with at least one actionable ticket prints the ticket id, state, and title of the candidate that would be dispatched next

### Out of scope

- Changing how `apm work` (non-dry-run) dispatches tickets
- Respecting `max_concurrent` in the dry-run output (the command currently shows all candidates; this ticket changes to showing only the next one)
- Any changes to `ticket::pick_next` signature or behaviour
- Adding a `pick_all` / `candidates_sorted` helper to `apm-core`

### Approach

**File changed:** `apm/src/cmd/work.rs` — `run_dry` function only.

Replace the current inline filter + sort + collect with a single call to
`ticket::pick_next`, matching the pattern used in `next.rs` and `start.rs`:

1. Build `startable` and `actionable` the same way as today (no change).
2. Load tickets with `ticket::load_all_from_git` (no change).
3. Call `ticket::pick_next(&tickets, &actionable, &startable, pw, ew, rw)`
   instead of the hand-rolled filter/sort loop.
4. Match on `Some(t)` / `None`:
   - `None` → print "dry-run: no actionable tickets" and return `Ok(())`.
   - `Some(t)` → print "dry-run: would start next: #id [state] title".

The `fm.agent.is_none()` guard disappears automatically because `pick_next`
does not include it.

Output format changes from listing *all* candidates to listing *the single
next* candidate, consistent with what `apm work` would actually dispatch on
its first iteration.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:11Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:17Z | new | in_design | claude-0330-0245-main |
