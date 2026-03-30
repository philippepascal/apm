+++
id = 69
title = "apm start --next should dispatch spec-writing agents via command:start trigger"
state = "accepted"
priority = 5
effort = 1
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0069-apm-start-next-should-dispatch-spec-writ"
created_at = "2026-03-29T23:34:47.739616Z"
updated_at = "2026-03-30T00:50:24.571504Z"
+++

## Spec

### Problem

`apm start --next` (and `spawn_next_worker`) filters candidate tickets by looking for states that have a `trigger = "command:start"` transition. Currently only `ready → in_progress` has that trigger — the spec-writing transitions (`new → in_design` and `ammend → in_design`) both have `trigger = "manual"`.

This means `apm start --next` can never dispatch a spec-writing agent. `new` and `ammend` tickets are actionable for agents but the delegator skips them entirely. The full autonomous loop described in `TICKET_LIFECYCLE.md` — delegator finds next ticket, provisions worktree, spawns appropriate subagent — is broken for the spec phase.

The fix is two-part: change `new → in_design` and `ammend → in_design` to `trigger = "command:start"` in `apm.toml`, so `apm start --next` can see and claim them.

### Acceptance criteria

- [x] `new → in_design` transition in `apm.toml` has `trigger = "command:start"`
- [x] `ammend → in_design` transition in `apm.toml` has `trigger = "command:start"`
- [x] `apm start --next` returns a `new` ticket when no `ready` tickets exist and a `new` ticket is available
- [x] `apm start --next` returns an `ammend` ticket when it is the highest-priority actionable ticket
- [x] Priority ordering still holds: a `ready` ticket with higher score beats a `new` ticket with lower score
- [x] `apm state <id> in_design` (manual transition) still works for supervisors claiming tickets directly — changing the trigger does not remove the manual path
- [x] Integration test: a repo with one `new` ticket and no `ready` tickets — `apm start --next` transitions the `new` ticket to `in_design` and returns it

### Out of scope

- Changing the actor restrictions on these transitions (`actor = "agent"` stays)
- Changing any other trigger types
- Runtime behaviour of `apm start --next` beyond ticket selection (instructions, worktree, spawn — already implemented in #56)

### Approach

1. In `apm.toml`, change `trigger = "manual"` to `trigger = "command:start"` on:
   - The `[[workflow.states.transitions]]` under `new` where `to = "in_design"`
   - The `[[workflow.states.transitions]]` under `ammend` where `to = "in_design"`

2. No Rust code changes needed — `apm start --next` already filters by `command:start` and already reads `instructions` from the target state. Once the trigger is set, spec tickets become visible to the delegator automatically.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:34Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:34Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:35Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:54Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:00Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T00:50Z | implemented | accepted | apm |
