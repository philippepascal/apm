+++
id = "e1748434"
title = "apm-ui: 3-column resizable/hidable layout shell with Zustand"
state = "in_progress"
priority = 60
effort = 3
risk = 2
author = "apm"
agent = "40064"
branch = "ticket/e1748434-apm-ui-3-column-resizable-hidable-layout"
created_at = "2026-03-31T06:11:50.266948Z"
updated_at = "2026-03-31T23:23:59.682805Z"
+++

## Spec

### Problem

The workscreen is the main view of the apm UI. It has three columns: workerview (left), supervisorview (middle), and ticket detail (right). Each column must be resizable by dragging dividers, hidable via a toggle control, with the constraint that at least one column always remains visible.

Currently there is no workscreen component at all; only the blank page with a console-logging stub delivered by Step 3.

Global UI state is managed by a Zustand store: selectedTicketId, column visibility flags, and column widths. Downstream tickets (Steps 5-7) will read and write this store without prop-drilling.

This ticket delivers the structural shell only: three labelled empty panels that prove resize and hide/show all work before any data is layered in.

### Acceptance criteria

- [x] Three panels labelled WorkerView, SupervisorView, and TicketDetail render side-by-side on the workscreen with no content inside them
- [x] Dragging the divider between any two adjacent columns resizes them in real time
- [x] Each column has a toggle control (button or icon) that hides it when clicked
- [x] Hiding a column collapses it to zero width; clicking its toggle again restores it
- [x] Attempting to hide the last visible column has no effect (the column stays visible)
- [x] Column visibility state is held in the Zustand store and survives React re-renders without resetting
- [ ] The Zustand store exposes selectedTicketId (null by default) and column width percentages alongside the visibility flags
- [ ] npm run build in apm-ui/ exits 0 with no TypeScript errors
- [ ] cargo test --workspace passes after all UI source changes are in place

### Out of scope

- Rendering any real ticket data (swimlanes, ticket cards, worker queue) — those are Steps 5-7
- The ticket detail markdown viewer — Step 6
- Arrow key navigation across ticket cards — Step 6
- Persistence of column state across browser sessions (localStorage)
- Mobile or responsive layouts
- The editor/review screen — Step 9
- Authentication, CORS, or any backend changes

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

Column hide/show: react-resizable-panels supports collapsible panels via the collapsible and onCollapse props. Use these to drive the Zustand visibility flags on user-drag-to-zero; separately, the toggle button calls toggleColumn() directly.

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

- [x] Remove the Acceptance Criterion "Pressing Ctrl+1, Ctrl+2, Ctrl+3 moves keyboard focus to WorkerView, SupervisorView, and TicketDetail respectively" — column visibility has no keyboard shortcut (toolbar-only per keyboard spec)
- [x] Remove the corresponding keyboard shortcut useEffect from the Approach (Step 4 "Keyboard shortcut handler" paragraph)

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:20Z | new | in_design | philippepascal |
| 2026-03-31T06:23Z | in_design | specd | claude-0330-0001-spec1 |
| 2026-03-31T18:14Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:02Z | ammend | in_design | philippepascal |
| 2026-03-31T19:07Z | in_design | specd | claude-0331-1430-spec2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-03-31T23:23Z | ready | in_progress | philippepascal |