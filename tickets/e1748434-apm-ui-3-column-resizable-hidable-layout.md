+++
id = "e1748434"
title = "apm-ui: 3-column resizable/hidable layout shell with Zustand"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "64729"
branch = "ticket/e1748434-apm-ui-3-column-resizable-hidable-layout"
created_at = "2026-03-31T06:11:50.266948Z"
updated_at = "2026-03-31T06:20:11.319397Z"
+++

## Spec

### Problem

The workscreen is the main view of the apm UI. It has three columns: workerview (left), supervisorview (middle), and ticket detail (right). Each column must be resizable by dragging dividers, hidable via a toggle control, with the constraint that at least one column always remains visible.

Currently there is no workscreen component at all; only the blank page with a console-logging stub delivered by Step 3.

Global UI state is managed by a Zustand store: selectedTicketId, column visibility flags, and column widths. Downstream tickets (Steps 5-7) will read and write this store without prop-drilling.

This ticket delivers the structural shell only: three labelled empty panels that prove resize, hide/show, and keyboard focus switching all work before any data is layered in.

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
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:20Z | new | in_design | philippepascal |