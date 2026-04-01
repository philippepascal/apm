+++
id = "5e3b3632"
title = "apm-ui: remove dry-run preview panel from WorkerView"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "57546"
branch = "ticket/5e3b3632-apm-ui-remove-dry-run-preview-panel-from"
created_at = "2026-04-01T07:10:20.365066Z"
updated_at = "2026-04-01T07:15:53.148638Z"
+++

## Spec

### Problem

Commit fabfef3d introduced a DryRunPreview component that renders above the ticket queue in WorkerView when the dispatch engine is stopped. The component occupies vertical space in the left column and pushes the priority queue — the most actionable information in that panel — further down or out of view.

The dry-run preview shows which tickets would be dispatched next, but this information is low-value in day-to-day use: it duplicates what is already visible in the queue itself, requires a stopped engine to appear, and offers no action beyond a manual refresh. Keeping it in the UI creates visual clutter without proportionate benefit.

The fix is to remove the DryRunPreview component from WorkerView and delete the component file. The /api/work/dry-run endpoint and all backend logic should be left in place — they may prove useful for CLI or future tooling.

### Acceptance criteria

- [ ] DryRunPreview.tsx no longer exists in apm-ui/src/components/
- [ ] WorkerView no longer imports DryRunPreview
- [ ] WorkerView renders no dry-run preview panel when the engine is stopped
- [ ] The ticket queue is visible at the top of the left column when the engine is stopped (no panel above it)
- [ ] The /api/work/dry-run HTTP endpoint continues to respond (backend unchanged)
- [ ] The apm-ui build completes without errors or unused-import warnings

### Out of scope

- Removing or modifying the /api/work/dry-run backend endpoint or its handler logic
- Any changes to other WorkerView sub-components (WorkerActivityPanel, queue rendering, etc.)
- Redesigning the left-column layout beyond removing the panel
- Adding an alternative surface for dry-run information (e.g. tooltip, CLI command)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:10Z | — | new | philippepascal |
| 2026-04-01T07:15Z | new | in_design | philippepascal |