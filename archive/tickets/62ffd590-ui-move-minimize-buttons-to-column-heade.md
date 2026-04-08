+++
id = "62ffd590"
title = "UI: move minimize buttons to column header, show icon in top-left only when minimized"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "29387"
branch = "ticket/62ffd590-ui-move-minimize-buttons-to-column-heade"
created_at = "2026-04-02T18:22:08.087696Z"
updated_at = "2026-04-02T20:43:04.847753Z"
+++

## Spec

### Problem

Currently, the three-panel layout (Workers, Board, Detail) has a shared top toolbar in WorkScreen.tsx (lines 180–191) that renders minimize/toggle buttons for each column. This means the controls are spatially disconnected from the panels they control — the user must scan up to the toolbar to hide a panel, and the toolbar icons give no visible indication of which panel is active or where each column lives.

When a column is collapsed, it disappears entirely (collapsedSize={0}), leaving no affordance to re-expand it other than the dim icon in the detached toolbar. There is no in-column header button and no contextual icon anchored to the collapsed position.

The desired behaviour: each column's own header contains its minimize button; when the column is collapsed it shrinks to a narrow strip showing only the column's icon at the top, giving users a clear, in-place target to re-expand. The top toolbar is removed.

### Acceptance criteria

- [x] Each column header (Workers, Board, Detail) contains a minimize button (icon-only) that collapses that column when clicked
- [x] Clicking the minimize button on an expanded column collapses it to a narrow strip
- [x] The narrow strip shows only the column's icon (Activity / Columns / FileText) aligned to the top-left
- [x] Clicking the icon in the collapsed strip re-expands the column
- [x] The top toolbar with the three global toggle buttons is removed
- [x] Columns that are collapsed via the handle (drag to zero) still show the icon strip and can be re-expanded by clicking it
- [x] All three columns (Workers, Board, Detail) behave consistently with this pattern

### Out of scope

- Log panel collapse behaviour (already has its own in-header toggle; unchanged)
- Keyboard shortcuts for toggling columns
- Persisting collapsed state across page reloads (store already handles this; no changes needed)
- ReviewEditor mode layout (the two-panel layout used when reviewMode=true is unchanged)
- Resizing columns — only the minimize/expand affordance changes

### Approach

Files changed: apm-ui/src/components/WorkScreen.tsx, WorkerView.tsx, SupervisorView.tsx, TicketDetail.tsx

**WorkScreen.tsx**
1. Remove the top toolbar div (lines 180–191) entirely.
2. Change each ResizablePanel: set collapsedSize to a small non-zero value (e.g. 3) so the collapsed strip has visible width. Keep minSize={10} unchanged.
3. Change the panel render logic: instead of rendering null when !columnVisibility[key], render a narrow strip div (h-full, flex flex-col, items-center) that shows only the column's Icon at the top (e.g. pt-2). Clicking the icon calls handleToggle(key).
4. Pass a handleToggle callback or the column key into each CONTENT component so the header can call it. Simplest: change CONTENT from a plain object to a function that accepts onMinimize, and pass handleToggle(key) as a prop.

**Column components (WorkerView, SupervisorView, TicketDetail)**
5. Each component accepts an onMinimize?: () => void prop.
6. Add a small icon button (ChevronLeft or Minimize2 from lucide-react) at the right side of the column header div. On click it calls onMinimize(). Apply a hover style consistent with the existing toolbar buttons (hover:bg-gray-700, text-gray-400).
   - WorkerView: the header at line 8 already has justify-between; add the button after WorkEngineControls.
   - SupervisorView: locate the top header bar and add the button there.
   - TicketDetail: locate or add a header bar and add the button there.

**Icon in collapsed strip**
The collapsed strip div in WorkScreen renders: Icon (w-4 h-4, text-gray-400) inside a button that calls handleToggle(key). The strip should be narrow (the 3% collapsedSize) and show the icon at the top (pt-2).

**handleResize in WorkScreen**
The existing handleResize already syncs the store when the panel is dragged to 0. It should continue working with the new collapsedSize=3 since the logic checks size.asPercentage === 0 (collapsed by drag) vs store state. No change needed to this logic — collapsed-by-drag will still set the column as not visible, and the panel will render the icon strip.

Wait — collapsedSize=3 means the panel collapses TO 3%, not 0. The onResize callback checks size.asPercentage === 0. We need to update handleResize to also treat size.asPercentage <= collapsedSize as collapsed. Change the condition to: isCollapsed = size.asPercentage <= 3.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T18:22Z | — | new | apm |
| 2026-04-02T18:22Z | new | groomed | apm |
| 2026-04-02T18:23Z | groomed | in_design | philippepascal |
| 2026-04-02T18:26Z | in_design | specd | claude-0402-1830-s9k2 |
| 2026-04-02T19:18Z | specd | ready | apm |
| 2026-04-02T19:34Z | ready | in_progress | philippepascal |
| 2026-04-02T19:38Z | in_progress | implemented | claude-0402-1940-w7x3 |
| 2026-04-02T20:43Z | implemented | closed | apm-sync |