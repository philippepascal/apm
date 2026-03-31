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

Prerequisites: Step 3 (ticket ed5c2b3b) must be implemented so apm-ui/ exists with Vite + React + TypeScript + shadcn/ui + TanStack Query in place.

**1. Install dependencies**

In apm-ui/:
  npm install zustand
  npx shadcn@latest add resizable

The shadcn resizable component wraps react-resizable-panels and provides ResizablePanelGroup, ResizablePanel, and ResizableHandle — consistent with the existing shadcn/ui design system already set up in Step 3.

**2. Zustand store — apm-ui/src/store/useLayoutStore.ts**

Create a store with:
  - selectedTicketId: string | null  (null by default)
  - columnVisibility: { workerView: boolean; supervisorView: boolean; ticketDetail: boolean }  (all true by default)
  - columnSizes: [number, number, number]  (e.g. [25, 50, 25] as percentages)
  - setSelectedTicketId(id: string | null): void
  - toggleColumn(col: 'workerView' | 'supervisorView' | 'ticketDetail'): void
    -- guard: if toggling would leave all three false, do nothing
  - setColumnSizes(sizes: [number, number, number]): void

**3. Column placeholder components**

Create apm-ui/src/components/WorkerView.tsx, SupervisorView.tsx, TicketDetail.tsx.
Each is a div with a header showing its name and a note that it is empty (e.g. light grey background, centred label). These are pure presentational stubs.

Each component wraps itself in a div with tabIndex={0} so it can receive keyboard focus.

**4. WorkScreen layout — apm-ui/src/components/WorkScreen.tsx**

Use ResizablePanelGroup with direction=horizontal containing three ResizablePanel / ResizableHandle pairs.

Each panel:
  - Reads its visibility flag from the store; when hidden, renders nothing and sets its minSize/maxSize to 0
  - Has a collapse toggle button in its header (an eye icon from lucide-react or a simple X)
  - Ref-forwards its focusable wrapper for Ctrl+1/2/3 shortcut targeting

Column hide/show: react-resizable-panels supports collapsible panels via the collapsible and onCollapse props. Use these to drive the Zustand visibility flags on user-drag-to-zero; separately, the toggle button calls toggleColumn() directly.

Keyboard shortcut handler: add a useEffect in WorkScreen that listens for keydown on document. On Ctrl+1/Ctrl+2/Ctrl+3, call .focus() on the ref for the corresponding panel (if visible); skip to the next visible panel if the target is hidden.

**5. Wire into App**

Replace the stub in apm-ui/src/App.tsx with:
  import WorkScreen from './components/WorkScreen'
  function App() { return <WorkScreen /> }

The existing TanStack Query useQuery console.log can be moved inside WorkScreen or dropped (Step 3's concern is satisfied; no need to keep the console.log in Step 4 onward).

**6. File changes summary**

New files:
  apm-ui/src/store/useLayoutStore.ts
  apm-ui/src/components/WorkScreen.tsx
  apm-ui/src/components/WorkerView.tsx
  apm-ui/src/components/SupervisorView.tsx
  apm-ui/src/components/TicketDetail.tsx
  apm-ui/src/components/ui/resizable.tsx  (added by shadcn CLI)

Modified files:
  apm-ui/src/App.tsx  (render WorkScreen instead of stub)
  apm-ui/package.json  (zustand added; react-resizable-panels added via shadcn)

No Rust / backend files change.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:20Z | new | in_design | philippepascal |