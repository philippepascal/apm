+++
id = "268f5694"
title = "apm-ui: ticket detail panel with markdown viewer and keyboard navigation"
state = "closed"
priority = 50
effort = 4
risk = 3
author = "apm"
agent = "30367"
branch = "ticket/268f5694-apm-ui-ticket-detail-panel-with-markdown"
created_at = "2026-03-31T06:12:10.547637Z"
updated_at = "2026-04-01T04:54:40.232407Z"
+++

## Spec

### Problem

The right column (TicketDetail) is a labelled placeholder stub delivered by Step 4. There is no way to view full ticket content in the UI — supervisors must leave the browser and use the CLI. This ticket wires up the detail view and global arrow-key navigation across the swimlane columns.

**Current state (after Step 5):**
- TicketDetail.tsx is a stub component with a centred label and no data
- SupervisorView renders swimlanes; clicking a card sets selectedTicketId in Zustand
- GET /api/tickets/:id exists and returns frontmatter + body as JSON

**Desired state:**
- TicketDetail fetches the selected ticket and renders its body as formatted, read-only markdown
- The detail panel updates reactively whenever selectedTicketId changes
- Arrow keys navigate selection across the swimlane grid (Left/Right between columns, Up/Down within a column), updating selectedTicketId as they go
- The newly-selected card scrolls into view automatically

**Who is affected:** Every person using the supervisor view — they need to read full ticket specs without switching to the CLI.

### Acceptance criteria

- [x] When selectedTicketId is null, TicketDetail shows a placeholder message
- [x] When a ticket is selected, TicketDetail fetches GET /api/tickets/:id via TanStack Query and renders the body as formatted markdown
- [x] Markdown rendering includes GFM: tables, strikethrough, task-list checkboxes, fenced code blocks
- [x] While the ticket is loading, TicketDetail shows a loading skeleton
- [x] If the fetch fails, TicketDetail shows an error message with the status code
- [x] The detail view updates automatically within one query-cache cycle when selectedTicketId changes
- [x] Pressing ArrowRight moves selection to the first card of the next swimlane column
- [x] Pressing ArrowRight on the last swimlane column has no effect
- [x] Pressing ArrowLeft moves selection to the first card of the previous swimlane column
- [x] Pressing ArrowLeft on the first swimlane column has no effect
- [x] Pressing ArrowDown moves selection to the next card within the current swimlane
- [x] Pressing ArrowDown on the last card of a swimlane has no effect
- [x] Pressing ArrowUp moves selection to the previous card within the current swimlane
- [x] Pressing ArrowUp on the first card of a swimlane has no effect
- [x] If no ticket is selected, pressing any arrow key selects the first card of the first visible swimlane
- [x] Arrow key events are ignored when event target is an input, textarea, select, or contenteditable element
- [x] Arrow key events are ignored when Ctrl or Meta is held
- [x] When keyboard navigation changes selection, the newly-selected card scrolls into view
- [x] npm run build in apm-ui/ exits 0 with no TypeScript errors
- [x] cargo test --workspace passes

### Out of scope

- Navigation into the worker queue (left column); Step 7 will extend the keyboard nav to include those tickets
- Editing ticket content — covered by Step 9
- State transition buttons on the detail panel — covered by Step 8
- The review/editor screen — covered by Step 9
- Persistence of selected ticket across browser sessions
- Keyboard navigation within the WorkerView panel (left column) — Step 7

### Approach

Prerequisites: Step 5 (ticket 3b0019a3) must be implemented so SupervisorView renders swimlanes and selectedTicketId is wired in Zustand.

**1. Install dependencies** (in apm-ui/)

  npm install react-markdown remark-gfm
  npm install -D @tailwindcss/typography

Add `@tailwindcss/typography` to the Tailwind plugins array in `tailwind.config.ts`.

**2. Markdown viewer — apm-ui/src/components/TicketDetail.tsx**

Replace the stub with a full component:

- Read `selectedTicketId` from the Zustand store (useLayoutStore)
- When null: render a centred grey placeholder ("Select a ticket to view details")
- When non-null: run a TanStack Query:
    useQuery({ queryKey: ['ticket', selectedTicketId], queryFn: () => fetch(`/api/tickets/${selectedTicketId}`).then(r => { if (!r.ok) throw r; return r.json(); }), enabled: !!selectedTicketId })
- Loading state: render a shadcn Skeleton filling the panel (a few lines of varying width)
- Error state: render an error card showing the HTTP status code
- Success: render `<ReactMarkdown remarkPlugins={[remarkGfm]}>{ticket.body}</ReactMarkdown>` inside a `<div className="prose prose-sm max-w-none overflow-y-auto p-4 h-full">`

The ticket JSON shape from GET /api/tickets/:id must include at minimum `id`, `title`, `body` (full markdown string). If the backend returns frontmatter + body separately, concatenate them for display.

**3. Shared grouping utility — apm-ui/src/lib/supervisorUtils.ts** (new file)

Extract the supervisor-state grouping logic into a reusable function so both SupervisorView and the keyboard nav handler share the same source of truth:

```ts
export const SUPERVISOR_STATES = ['question', 'specd', 'blocked', 'implemented', 'accepted'] as const;
export type SupervisorState = typeof SUPERVISOR_STATES[number];

export function groupBySupervisorState(tickets: Ticket[]): [SupervisorState, Ticket[]][] {
  // returns ordered pairs, omitting states with no tickets
}
```

Update SupervisorView.tsx to import from this utility instead of duplicating logic.

**4. Keyboard navigation — apm-ui/src/components/WorkScreen.tsx**

Add a `useEffect` that attaches a `keydown` listener to `document`. The handler:

1. Returns early if `event.ctrlKey || event.metaKey`
2. Returns early if `['ArrowUp','ArrowDown','ArrowLeft','ArrowRight'].indexOf(event.key) === -1`
3. Returns early if `event.target` matches `input, textarea, select, [contenteditable]`
4. Calls `event.preventDefault()` to suppress browser scroll
5. Reads `selectedTicketId` and the current `tickets` query result from TanStack Query cache
6. Calls `groupBySupervisorState(tickets)` to get the ordered grid as `columns: [state, Ticket[]][]`
7. Finds the current ticket's column index and row index; if not found (no selection), selects `columns[0][1][0]` and returns
8. Computes new position based on key:
    - ArrowRight: colIdx + 1 (if within bounds), rowIdx = 0
    - ArrowLeft: colIdx - 1 (if within bounds), rowIdx = 0
    - ArrowDown: rowIdx + 1 within same column (if within bounds)
    - ArrowUp: rowIdx - 1 within same column (if within bounds)
9. Calls `setSelectedTicketId(newTicket.id)` on the store
10. Calls `document.querySelector('[data-ticket-id="' + newId + '"]')?.scrollIntoView({ block: 'nearest' })`

The useEffect cleanup removes the listener on unmount.

**5. TicketCard data attribute — apm-ui/src/components/supervisor/TicketCard.tsx**

Add `data-ticket-id={ticket.id}` to the root element of TicketCard so the scroll-into-view selector can find it.

**6. File changes summary**

New files:
  apm-ui/src/lib/supervisorUtils.ts

Modified files:
  apm-ui/src/components/TicketDetail.tsx      (replace stub with markdown viewer)
  apm-ui/src/components/WorkScreen.tsx        (add keyboard nav handler)
  apm-ui/src/components/supervisor/SupervisorView.tsx   (import from supervisorUtils)
  apm-ui/src/components/supervisor/TicketCard.tsx       (add data-ticket-id attribute)
  apm-ui/tailwind.config.ts                   (add typography plugin)
  apm-ui/package.json                         (react-markdown, remark-gfm, @tailwindcss/typography)

No Rust / backend files change.

**7. Extension point for Step 7**

Step 7 will add the worker queue. The keyboard nav in WorkScreen should be designed so that WorkerView can register its ordered ticket list. The simplest seam: after Step 6, when ArrowLeft is pressed from column 0 in the swimlanes, nothing happens. Step 7 will update this handler to continue into the worker queue list.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:28Z | new | in_design | philippepascal |
| 2026-03-31T06:33Z | in_design | specd | claude-0330-spec-a7f2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-04-01T00:53Z | ready | in_progress | philippepascal |
| 2026-04-01T00:58Z | in_progress | implemented | claude-0401-0100-w268 |
| 2026-04-01T01:22Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |