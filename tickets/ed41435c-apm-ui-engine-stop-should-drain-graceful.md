+++
id = "ed41435c"
title = "apm-ui: engine stop should drain gracefully; individual stop should kill"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "88851"
branch = "ticket/ed41435c-apm-ui-engine-stop-should-drain-graceful"
created_at = "2026-04-01T07:56:48.702217Z"
updated_at = "2026-04-01T07:57:24.298225Z"
+++

## Spec

### Problem

The UI has two distinct "stop" actions, but neither communicates its actual semantics to the user:

1. **Engine Stop** (`WorkEngineControls`, POST `/api/work/stop`): The backend correctly drains — it sets a cancel flag on the dispatch loop so no new workers are started, but running workers are left to finish their current ticket. The button label is simply "Stop", giving the impression the engine (and all workers) are immediately halted. Users who click it expect work to cease instantly; instead workers keep running silently.

2. **Individual worker Stop** (`WorkerActivityPanel`, DELETE `/api/workers/:pid`): The backend sends SIGTERM to the specific worker process. The button label "Stop" is accurate in effect but shares identical wording with the engine Stop, blurring the distinction between "drain" and "kill".

The fix is entirely in the UI layer. Backend behaviour is already correct. Clarifying the labels and adding `title` tooltips will eliminate user confusion about what each action does and whether data/work is at risk.

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
| 2026-04-01T07:56Z | — | new | philippepascal |
| 2026-04-01T07:57Z | new | in_design | philippepascal |