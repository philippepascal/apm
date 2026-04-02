+++
id = "c1ff90de"
title = "Add depends_on scheduling to engine dispatch loop"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "59408"
branch = "ticket/c1ff90de-add-depends-on-scheduling-to-engine-disp"
created_at = "2026-04-01T21:55:02.787625Z"
updated_at = "2026-04-02T00:43:15.939908Z"
+++

## Spec

### Problem

Once `depends_on` is stored in ticket frontmatter (ticket d877bd37), the engine dispatch loop must honour it. Currently `pick_next` returns the highest-priority actionable ticket unconditionally — neither the dispatch loop in `spawn_next_worker` nor the `apm next` command has any awareness of ticket dependencies.

The full design is in `docs/epics.md` (§ depends_on scheduling — Engine loop change). Before a candidate ticket is dispatched, every entry in its `depends_on` list must be checked: if any referenced ticket exists and is not yet in state `implemented` or later, the candidate must be skipped and the next highest-scoring non-blocked ticket tried instead. An unknown dep ID (no matching ticket found) is treated as non-blocking. The check is config-driven: "implemented or later" means the dep ticket's state appears at the same position or later than `implemented` in `config.workflow.states`, or the dep ticket's state has `terminal = true`.

### Acceptance criteria

- [ ] When a ticket has `depends_on = ["<id>"]` and the referenced ticket is in a state before `implemented`, `spawn_next_worker` skips it and dispatches the next highest-priority non-blocked ticket instead
- [ ] When all entries in `depends_on` are in state `implemented` or later, the ticket is eligible for dispatch as normal
- [ ] A state with `terminal = true` satisfies the dependency check regardless of its position in the workflow states list
- [ ] A `depends_on` entry whose ID does not match any known ticket is treated as non-blocking (the candidate is not skipped due to that entry)
- [ ] A ticket with an empty `depends_on = []` is treated identically to a ticket with no `depends_on` field
- [ ] `apm next` skips dep-blocked tickets by the same rule — it returns the highest-scoring ticket whose deps are all satisfied
- [ ] The dep-blocking logic does not hardcode state names beyond `implemented` as the threshold; states that appear after `implemented` in the workflow states list also satisfy the check

### Out of scope

- Adding `depends_on` to `Frontmatter` — that is ticket d877bd37
- UI lock icon on ticket cards (separate UI ticket per the epic design)
- Circular dependency detection or warnings
- Changes to `apm list` output to surface blocked tickets
- `apm work --dry-run` output (ticket 18c00750 covers that separately)
- Epic-scoped filtering of the dispatch queue (separate ticket in the epic)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |