+++
id = 45
title = "need a new state called in_design"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "apm"
agent = "claude-0328-1430-a4f2"
branch = "ticket/0045-need-a-new-state-called-in-design"
created_at = "2026-03-28T09:04:04.270348Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

When an agent picks up a `new` or `ammend` ticket to write or revise a spec, the ticket stays in `new`/`ammend` until the agent finishes and runs `apm state <id> specd`. Supervisors have no visibility into which tickets are actively being specced versus sitting idle. Adding an `in_design` state gives agents a place to claim a ticket during spec-writing, mirroring how `in_progress` works during implementation.

### Acceptance criteria

- [x] `in_design` state exists in `apm.toml` with `actionable = ["agent"]` and `layer = 1`
- [x] Transition `new` → `in_design` is allowed (actor: agent, manual)
- [x] Transition `ammend` → `in_design` is allowed (actor: agent, manual)
- [x] Transition `in_design` → `specd` is allowed (actor: agent, manual, preconditions: `spec_not_empty`, `spec_has_acceptance_criteria`)
- [x] Transition `in_design` → `question` is allowed (actor: agent, manual)
- [x] `apm state <id> in_design` succeeds from `new` or `ammend`
- [x] `apm list --state in_design` shows tickets in that state
- [x] `apm next` does not return `in_design` tickets as actionable (they are already claimed)
- [x] Agent instructions in `apm.agents.md` document the new state and when to use it

### Out of scope

- An automatic transition into `in_design` (agents trigger it manually, same as `specd`)
- Any UI or dashboard changes
- Changing the `in_progress` state or implementation workflow

### Approach

1. **`apm.toml`** — add `in_design` state block between `ammend` and `ready`:
   ```toml
   [[workflow.states]]
   id         = "in_design"
   label      = "In Design"
   color      = "#f97316"
   layer      = 1
   actionable = ["agent"]

     [[workflow.states.transitions]]
     to            = "specd"
     trigger       = "manual"
     actor         = "agent"
     preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

     [[workflow.states.transitions]]
     to      = "question"
     trigger = "manual"
     actor   = "agent"
   ```
   Then add a `to = "in_design"` transition under the `new` state block and under the `ammend` state block (actor: agent, manual).

2. **`apm.agents.md`** — update the "Working a ticket" section:
   - Under `state = "new"`: after reading the ticket, add `apm state <id> in_design` before editing the spec file.
   - Under `state = "ammend"`: same — transition to `in_design` before editing.
   - Add a short paragraph describing `in_design` as the state that signals "agent is actively writing/revising the spec".
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T09:04Z | — | new | apm |
| 2026-03-28T18:27Z | new | specd | claude-0328-c72b |
| 2026-03-28T22:01Z | specd | ready | apm |
| 2026-03-28T22:03Z | ready | in_progress | claude-0328-1430-a4f2 |
| 2026-03-28T22:05Z | in_progress | ready | claude-0328-1430-a4f2 |
| 2026-03-28T22:06Z | ready | in_progress | claude-0328-1430-a4f2 |
| 2026-03-28T22:09Z | in_progress | implemented | claude-0328-1430-a4f2 |
| 2026-03-28T22:22Z | implemented | accepted | claude-0328-1430-a4f2 |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |