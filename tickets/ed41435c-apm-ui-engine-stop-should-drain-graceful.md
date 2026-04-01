+++
id = "ed41435c"
title = "apm-ui: engine stop should drain gracefully; individual stop should kill"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/ed41435c-apm-ui-engine-stop-should-drain-graceful"
created_at = "2026-04-01T07:56:48.702217Z"
updated_at = "2026-04-01T07:56:48.702217Z"
+++

## Spec

### Problem

There are two stop actions in the UI that should have distinct semantics:

1. WorkEngineControls 'Stop' button (POST /api/work/stop): should stop the engine from dispatching new workers, but leave currently running workers alive to finish. This is already what the backend does (sets cancel flag on the dispatch loop, does not signal worker processes). However the UI gives no indication of this — the engine shows 'stopped' immediately while workers may still be running. The Stop button label and/or tooltip should communicate 'stop dispatching' rather than 'kill all workers'.

2. WorkerActivityPanel individual 'Stop' button (DELETE /api/workers/:pid): should kill that specific worker process with SIGTERM. This is already what the backend does.

The fix is primarily UX/labeling:
- Rename or add a tooltip to the engine Stop button to make clear it stops dispatching, not workers (e.g. 'Stop dispatching' or tooltip 'Running workers will finish their current ticket')
- The individual Stop button in WorkerActivityPanel should be clearly labeled as a kill action (e.g. 'Kill' or kept as 'Stop' but with tooltip 'Send SIGTERM to this worker')
- No backend changes required — the semantics are already correct, the UI just does not communicate them.

What is broken or missing, and why it matters.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:56Z | — | new | philippepascal |
