+++
id = "e1c4ca76"
title = "UI Workers list sorts tickets by worker status, with crashed and ended last"
state = "ready"
priority = 0
effort = 1
risk = 1
author = "apm-ui"
branch = "ticket/e1c4ca76-ui-workers-list-sorts-tickets-by-worker-"
created_at = "2026-04-04T03:16:37.369960Z"
updated_at = "2026-04-04T07:15:57.145980Z"
+++

## Spec

### Problem

The Workers list in the UI (WorkerActivityPanel.tsx) renders workers in the order returned by the API — currently unordered. When there are a mix of running, crashed, and ended workers, active workers can appear below idle ones, making it hard to quickly spot what is running.

The desired behaviour is that running workers always appear at the top, and crashed/ended workers are pushed to the bottom. This improves at-a-glance status awareness for the supervisor.

### Acceptance criteria

- [ ] Workers with status "running" appear before workers with status "crashed" in the list
- [ ] Workers with status "running" appear before workers with status "ended" in the list
- [ ] Workers with status "crashed" appear before workers with status "ended" in the list
- [ ] Within the same status group, relative order is stable (workers in the same status group maintain their original API order)
- [ ] When all workers share the same status, the list order is unchanged from the API response

### Out of scope

- Sorting by any field other than status (e.g. elapsed time, agent name, ticket title)
- User-configurable sort order
- Sorting the priority queue panel (PriorityQueuePanel.tsx)
- Any changes to the server-side /api/workers response

### Approach

**File:** apm-ui/src/components/WorkerActivityPanel.tsx

**Change:** Sort the workers array before rendering, using a status priority map.

Define a priority order:
```ts
const STATUS_ORDER: Record<WorkerInfo['status'], number> = {
  running: 0,
  crashed: 1,
  ended:   2,
}
```

In the render path, replace the bare `data.map()` call with a sorted copy:
```ts
const sorted = [...data].sort((a, b) => STATUS_ORDER[a.status] - STATUS_ORDER[b.status])
```

Then render `sorted.map()` instead of `data.map()`.

The sort is stable in all modern JS runtimes (ES2019+, and Vite's target), so workers within the same status group keep their original API order.

No other files change. No new state, hooks, or deps needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T03:16Z | — | new | apm-ui |
| 2026-04-04T06:02Z | new | groomed | apm |
| 2026-04-04T06:40Z | groomed | in_design | philippepascal |
| 2026-04-04T06:42Z | in_design | specd | claude-0403-spec-e1c4 |
| 2026-04-04T07:15Z | specd | ready | apm |
