+++
id = "4223716a"
title = "apm start --next should dispatch spec-writer agent for new/ammend tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/4223716a-apm-start-next-should-dispatch-spec-writ"
created_at = "2026-03-30T20:52:32.229319Z"
updated_at = "2026-03-30T20:52:54.638908Z"
+++

## Spec

### Problem

`apm start --next` always dispatches a worker using `.apm/worker.md` as the system prompt, regardless of the ticket's state. This means spec-writing work (tickets in `new` or `ammend` state) is handed to the same implementation-focused worker agent.

`.apm/spec-writer.md` exists specifically for this purpose — a different system prompt tuned for writing specs, assessing effort/risk, and asking clarifying questions — but it is never loaded. The distinction matters: a good spec-writer agent should be conservative, ask questions, and fill all four required sections; an implementation worker should be execution-focused.

`apm start` should select the system prompt based on the ticket's current state:
- `new` or `ammend` → use `.apm/spec-writer.md` (fall back to `.apm/worker.md` if absent)
- all other startable states → use `.apm/worker.md`

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T20:52Z | — | new | philippepascal |
| 2026-03-30T20:52Z | new | in_design | philippepascal |
