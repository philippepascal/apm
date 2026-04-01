+++
id = "5e3b3632"
title = "apm-ui: remove dry-run preview panel from WorkerView"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/5e3b3632-apm-ui-remove-dry-run-preview-panel-from"
created_at = "2026-04-01T07:10:20.365066Z"
updated_at = "2026-04-01T07:15:53.148638Z"
+++

## Spec

### Problem

fabfef3d added a DryRunPreview component that renders above the queue in WorkerView when the engine is stopped. In practice it takes up space in the left column and hides the priority queue, which is more useful. The dry-run information (what would be dispatched next) is not actionable enough to warrant a persistent panel in the UI. Remove DryRunPreview from WorkerView entirely. The /api/work/dry-run endpoint and its backend logic can stay — it may be useful later — but the UI component should be deleted and its import removed from WorkerView.

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
| 2026-04-01T07:10Z | — | new | philippepascal |
| 2026-04-01T07:15Z | new | in_design | philippepascal |
