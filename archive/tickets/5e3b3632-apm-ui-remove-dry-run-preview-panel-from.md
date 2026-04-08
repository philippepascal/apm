+++
id = "5e3b3632"
title = "apm-ui: remove dry-run preview panel from WorkerView"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "46964"
branch = "ticket/5e3b3632-apm-ui-remove-dry-run-preview-panel-from"
created_at = "2026-04-01T07:10:20.365066Z"
updated_at = "2026-04-01T07:47:35.423936Z"
+++

## Spec

### Problem

Commit fabfef3d introduced a DryRunPreview component that renders above the ticket queue in WorkerView when the dispatch engine is stopped. The component occupies vertical space in the left column and pushes the priority queue — the most actionable information in that panel — further down or out of view.

The dry-run preview shows which tickets would be dispatched next, but this information is low-value in day-to-day use: it duplicates what is already visible in the queue itself, requires a stopped engine to appear, and offers no action beyond a manual refresh. Keeping it in the UI creates visual clutter without proportionate benefit.

The fix is to remove the DryRunPreview component from WorkerView and delete the component file. The /api/work/dry-run endpoint and all backend logic should be left in place — they may prove useful for CLI or future tooling.

### Acceptance criteria

- [x] DryRunPreview.tsx no longer exists in apm-ui/src/components/
- [x] WorkerView no longer imports DryRunPreview
- [x] WorkerView renders no dry-run preview panel when the engine is stopped
- [x] The ticket queue is visible at the top of the left column when the engine is stopped (no panel above it)
- [x] The /api/work/dry-run HTTP endpoint continues to respond (backend unchanged)
- [x] The apm-ui build completes without errors or unused-import warnings

### Out of scope

- Removing or modifying the /api/work/dry-run backend endpoint or its handler logic
- Any changes to other WorkerView sub-components (WorkerActivityPanel, queue rendering, etc.)
- Redesigning the left-column layout beyond removing the panel
- Adding an alternative surface for dry-run information (e.g. tooltip, CLI command)

### Approach

Two files change; no new files are created.

1. **Delete** apm-ui/src/components/DryRunPreview.tsx — the entire file is removed.

2. **Edit** apm-ui/src/components/WorkerView.tsx:
   - Remove the import line: `import DryRunPreview from './DryRunPreview'`
   - Remove the `<DryRunPreview />` JSX element (currently rendered unconditionally inside the flex container, between the border divider and the Queue section)
   - No other layout changes are needed; the surrounding flex container will naturally expand to fill the reclaimed space.

3. Run `npm run build` (or equivalent) inside apm-ui to confirm the build is clean.

No backend files change. No new tests are needed (there were none for DryRunPreview, and the component is being deleted, not modified).

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:10Z | — | new | philippepascal |
| 2026-04-01T07:15Z | new | in_design | philippepascal |
| 2026-04-01T07:17Z | in_design | specd | claude-0401-0715-15f0 |
| 2026-04-01T07:21Z | specd | ready | apm |
| 2026-04-01T07:23Z | ready | in_progress | philippepascal |
| 2026-04-01T07:26Z | in_progress | implemented | claude-0401-0723-71c0 |
| 2026-04-01T07:46Z | implemented | accepted | apm-sync |
| 2026-04-01T07:47Z | accepted | closed | apm-sync |