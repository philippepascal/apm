+++
id = "c1ff90de"
title = "Add depends_on scheduling to engine dispatch loop"
state = "closed"
priority = 8
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "77015"
branch = "ticket/c1ff90de-add-depends-on-scheduling-to-engine-disp"
created_at = "2026-04-01T21:55:02.787625Z"
updated_at = "2026-04-02T05:24:59.127251Z"
+++

## Spec

### Problem

Once `depends_on` is stored in ticket frontmatter (ticket d877bd37), the engine dispatch loop must honour it. Currently `pick_next` returns the highest-priority actionable ticket unconditionally — neither the dispatch loop in `spawn_next_worker` nor the `apm next` command has any awareness of ticket dependencies.

The full design is in `docs/epics.md` (§ depends_on scheduling — Engine loop change). Before a candidate ticket is dispatched, every entry in its `depends_on` list must be checked: if any referenced ticket exists and is not yet in state `implemented` or later, the candidate must be skipped and the next highest-scoring non-blocked ticket tried instead. An unknown dep ID (no matching ticket found) is treated as non-blocking. The check is config-driven: "implemented or later" means the dep ticket's state appears at the same position or later than `implemented` in `config.workflow.states`, or the dep ticket's state has `terminal = true`.

### Acceptance criteria

- [x] When a ticket has `depends_on = ["<id>"]` and the referenced ticket's state has neither `satisfies_deps = true` nor `terminal = true`, `spawn_next_worker` skips it and dispatches the next highest-priority non-blocked ticket instead
- [x] When all entries in `depends_on` reference tickets whose states have `satisfies_deps = true` or `terminal = true`, the ticket is eligible for dispatch as normal
- [x] A state with `terminal = true` satisfies the dependency check regardless of its position in the workflow states list or its `satisfies_deps` value
- [x] A `depends_on` entry whose ID does not match any known ticket is treated as non-blocking (the candidate is not skipped due to that entry)
- [x] A ticket with an empty `depends_on = []` is treated identically to a ticket with no `depends_on` field
- [x] `apm next` skips dep-blocked tickets by the same rule — it returns the highest-scoring ticket whose deps are all satisfied
- [x] The dep-blocking check is driven entirely by `satisfies_deps` and `terminal` config flags — no state name is compared by string in the implementation

### Out of scope

- Adding `depends_on` to `Frontmatter` — that is ticket d877bd37
- UI lock icon on ticket cards (separate UI ticket per the epic design)
- Circular dependency detection or warnings
- Changes to `apm list` output to surface blocked tickets
- `apm work --dry-run` output (ticket 18c00750 covers that separately)
- Epic-scoped filtering of the dispatch queue (separate ticket in the epic)

### Approach

This ticket depends on d877bd37 landing first (adds `depends_on: Option<Vec<String>>` to `Frontmatter`).

**1. Add `satisfies_deps` to `StateConfig` — `apm-core/src/config.rs`**

Add a `satisfies_deps: bool` field (default `false`) to `StateConfig`, annotated with `#[serde(default)]`. This field signals that tickets in this state are considered "done enough" to unblock dependents. It is set independently of `terminal`; both flags independently satisfy the check.

**2. Add `dep_satisfied` helper — `apm-core/src/ticket.rs`**

Add a pub function `dep_satisfied(state: &str, config: &Config) -> bool`. It looks up the state in `config.workflow.states` and returns `true` if the matching `StateConfig` has `satisfies_deps = true` OR `terminal = true`. Returns `false` if the state is unknown or neither flag is set. No state name string comparisons beyond the lookup key.

**3. Extend `pick_next` signature — `apm-core/src/ticket.rs`**

Add `config: &crate::config::Config` parameter after `startable`. Inside the existing `find` closure, add dep-block filtering: for each `dep_id` in `t.frontmatter.depends_on`, look up the dep ticket in `tickets`; if found, call `dep_satisfied(dep_state, config)`; if it returns `false`, the candidate is blocked (`return false`). Unknown dep IDs (ticket not found) are non-blocking.

The existing iterator already tries candidates in score order; adding this filter means it naturally falls through to the next candidate.

**4. Update call sites**

Three locations call `ticket::pick_next` — all have `config` in scope:

- `apm-core/src/start.rs` line ~319 (non-aggressive `spawn_next_worker`): add `&config`
- `apm-core/src/start.rs` line ~474 (main `spawn_next_worker`): add `&config`
- `apm/src/cmd/next.rs` line ~20: add `&config`

**5. Tests**

Unit tests in `apm-core/src/ticket.rs`:
- `dep_satisfied` returns `true` for a state with `satisfies_deps = true`
- `dep_satisfied` returns `true` for a state with `terminal = true` (even if `satisfies_deps = false`)
- `dep_satisfied` returns `false` for a state with both flags `false`
- `dep_satisfied` returns `false` for an unknown state name
- `pick_next` skips a dep-blocked ticket and returns the next unblocked one
- `pick_next` returns a ticket once its dep reaches a `satisfies_deps = true` state
- `pick_next` does not skip a ticket with an unknown dep ID
- `pick_next` does not skip a ticket with an empty `depends_on` list

Integration test in `apm/tests/integration.rs`:
- Set up two tickets (A in `ready`, B in `ready` with `depends_on = [A.id]`); `apm next` returns A
- After A is transitioned to a `satisfies_deps = true` state, `apm next` returns B

### Open questions


### Amendment requests

- [x] "Implemented or later" must not be hardcoded. Replace throughout with: a dep is satisfied when the referenced ticket's state has `satisfies_deps = true` OR `terminal = true` in `config.workflow.states`. The `satisfies_deps: bool` field (default false) must be added to `StateConfig` in `apm-core/src/config.rs` as part of this ticket.
- [x] Rename `is_implemented_or_later` to `dep_satisfied(state: &str, config: &Config) -> bool` in all AC and Approach references.
- [x] Remove all mentions of the state name "implemented" from AC and Approach. Replace AC #1 with: "skips the ticket when any dep's state has neither `satisfies_deps = true` nor `terminal = true`". Replace AC #2 with: "returns the ticket when all dep states have `satisfies_deps = true` or `terminal = true`". Replace AC #7 with: "the check is driven entirely by `satisfies_deps` and `terminal` config flags — no state name is compared by string in the implementation".

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:47Z | in_design | specd | claude-0401-2200-s9w1 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:40Z | ammend | in_design | philippepascal |
| 2026-04-02T01:41Z | in_design | specd | claude-0402-0200-spec1 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T03:16Z | ready | in_progress | philippepascal |
| 2026-04-02T03:24Z | in_progress | implemented | claude-0402-0316-w7k2 |
| 2026-04-02T05:24Z | implemented | closed | apm-sync |