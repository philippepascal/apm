+++
id = "62ffd590"
title = "UI: move minimize buttons to column header, show icon in top-left only when minimized"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "19457"
branch = "ticket/62ffd590-ui-move-minimize-buttons-to-column-heade"
created_at = "2026-04-02T18:22:08.087696Z"
updated_at = "2026-04-02T18:23:53.682427Z"
+++

## Spec

### Problem

Currently, the three-panel layout (Workers, Board, Detail) has a shared top toolbar in WorkScreen.tsx (lines 180–191) that renders minimize/toggle buttons for each column. This means the controls are spatially disconnected from the panels they control — the user must scan up to the toolbar to hide a panel, and the toolbar icons give no visible indication of which panel is active or where each column lives.

When a column is collapsed, it disappears entirely (collapsedSize={0}), leaving no affordance to re-expand it other than the dim icon in the detached toolbar. There is no in-column header button and no contextual icon anchored to the collapsed position.

The desired behaviour: each column's own header contains its minimize button; when the column is collapsed it shrinks to a narrow strip showing only the column's icon at the top, giving users a clear, in-place target to re-expand. The top toolbar is removed.

### Acceptance criteria

- [ ] Each column header (Workers, Board, Detail) contains a minimize button (icon-only) that collapses that column when clicked
- [ ] Clicking the minimize button on an expanded column collapses it to a narrow strip
- [ ] The narrow strip shows only the column's icon (Activity / Columns / FileText) aligned to the top-left
- [ ] Clicking the icon in the collapsed strip re-expands the column
- [ ] The top toolbar with the three global toggle buttons is removed
- [ ] Columns that are collapsed via the handle (drag to zero) still show the icon strip and can be re-expanded by clicking it
- [ ] All three columns (Workers, Board, Detail) behave consistently with this pattern

### Out of scope

- Log panel collapse behaviour (already has its own in-header toggle; unchanged)
- Keyboard shortcuts for toggling columns
- Persisting collapsed state across page reloads (store already handles this; no changes needed)
- ReviewEditor mode layout (the two-panel layout used when reviewMode=true is unchanged)
- Resizing columns — only the minimize/expand affordance changes

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T18:22Z | — | new | apm |
| 2026-04-02T18:22Z | new | groomed | apm |
| 2026-04-02T18:23Z | groomed | in_design | philippepascal |