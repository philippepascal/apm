+++
id = "1099fe38"
title = "UI: add epic column and filter to queue panel"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "40989"
branch = "ticket/1099fe38-ui-add-epic-column-and-filter-to-queue-p"
created_at = "2026-04-01T21:56:20.710748Z"
updated_at = "2026-04-02T00:56:57.977427Z"
+++

## Spec

### Problem

The queue panel table in `PriorityQueuePanel.tsx` shows tickets without any indication of which epic they belong to, and provides no way to filter by epic. When multiple epics are in flight, tickets from all epics are interleaved and there is no way to focus on a single epic's work queue.

The fix is two additive changes to the queue panel: (1) an **Epic** column that shows the short 8-char epic ID for tickets inside an epic, or "—" for free tickets; and (2) an epic filter dropdown that hides tickets not belonging to the selected epic.

The `epic` field does not yet exist on `Frontmatter` (apm-core) or `QueueEntry` (apm-server), so both must be extended. The UI change is purely additive — no existing columns or interactions change.

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
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:56Z | groomed | in_design | philippepascal |