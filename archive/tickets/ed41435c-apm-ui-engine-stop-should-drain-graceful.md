+++
id = "ed41435c"
title = "apm-ui: engine stop should drain gracefully; individual stop should kill"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "43949"
branch = "ticket/ed41435c-apm-ui-engine-stop-should-drain-graceful"
created_at = "2026-04-01T07:56:48.702217Z"
updated_at = "2026-04-01T21:29:09.690001Z"
+++

## Spec

### Problem

The UI has two distinct "stop" actions, but neither communicates its actual semantics to the user:

1. **Engine Stop** (`WorkEngineControls`, POST `/api/work/stop`): The backend correctly drains — it sets a cancel flag on the dispatch loop so no new workers are started, but running workers are left to finish their current ticket. The button label is simply "Stop", giving the impression the engine (and all workers) are immediately halted. Users who click it expect work to cease instantly; instead workers keep running silently.

2. **Individual worker Stop** (`WorkerActivityPanel`, DELETE `/api/workers/:pid`): The backend sends SIGTERM to the specific worker process. The button label "Stop" is accurate in effect but shares identical wording with the engine Stop, blurring the distinction between "drain" and "kill".

The fix is entirely in the UI layer. Backend behaviour is already correct. Clarifying the labels and adding `title` tooltips will eliminate user confusion about what each action does and whether data/work is at risk.

### Acceptance criteria

- [x] The engine Stop button in WorkEngineControls displays the label "Stop dispatching" (or equivalent wording that references dispatching, not workers)
- [x] The engine Stop button has a `title` attribute reading "Running workers will finish their current ticket"
- [x] The individual worker Stop button in WorkerActivityPanel displays the label "Kill"
- [x] The individual worker Stop button has a `title` attribute reading "Send SIGTERM to this worker"
- [x] No behaviour change: clicking engine Stop still posts to `/api/work/stop`; clicking worker Kill still sends DELETE to `/api/workers/:pid`

### Out of scope

- Any backend changes — the drain/kill semantics are already correct
- Adding a confirmation dialog before killing a worker
- Adding a "drain and stop" feature that waits for all workers before marking the engine as stopped
- Changes to the engine status display or worker status indicators beyond button labels
- Accessibility improvements beyond `title` attributes (aria-live, keyboard nav, etc.)

### Approach

Two files change; both are minimal text/attribute edits.

**`apm-ui/src/components/WorkEngineControls.tsx`** (line ~77)

Change the engine toggle button so that when the engine is active it renders "Stop dispatching" with a `title` tooltip:

```tsx
<button
  onClick={handleToggle}
  disabled={isPending}
  title={isEngineActive ? 'Running workers will finish their current ticket' : undefined}
  className="px-2 py-0.5 rounded border border-gray-600 text-gray-300 text-xs hover:bg-gray-700 disabled:opacity-50"
>
  {isEngineActive ? 'Stop dispatching' : 'Start'}
</button>
```

**`apm-ui/src/components/WorkerActivityPanel.tsx`** (line ~121)

Change the per-worker stop button label to "Kill" and add a `title`:

```tsx
<button
  className="px-2 py-0.5 text-xs rounded bg-red-700 hover:bg-red-600 text-white disabled:opacity-50 shrink-0"
  disabled={stopping === w.pid}
  title="Send SIGTERM to this worker"
  onClick={() => handleStop(w.pid)}
>
  Kill
</button>
```

No other files need to change. No new dependencies. No backend changes. Tests: the project has no frontend unit tests covering button labels; verify manually by running the dev server and inspecting both panels.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:56Z | — | new | philippepascal |
| 2026-04-01T07:57Z | new | in_design | philippepascal |
| 2026-04-01T07:59Z | in_design | specd | claude-0401-0757-5cc0 |
| 2026-04-01T08:02Z | specd | ready | apm |
| 2026-04-01T08:02Z | ready | in_progress | philippepascal |
| 2026-04-01T08:05Z | in_progress | implemented | claude-0401-0802-33d0 |
| 2026-04-01T08:06Z | implemented | accepted | apm |
| 2026-04-01T21:29Z | accepted | closed | apm-sync |