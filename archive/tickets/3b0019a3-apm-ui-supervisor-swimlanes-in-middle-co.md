+++
id = "3b0019a3"
title = "apm-ui: supervisor swimlanes in middle column"
state = "closed"
priority = 55
effort = 3
risk = 2
author = "apm"
agent = "94914"
branch = "ticket/3b0019a3-apm-ui-supervisor-swimlanes-in-middle-co"
created_at = "2026-03-31T06:11:59.993473Z"
updated_at = "2026-04-01T04:54:49.252409Z"
+++

## Spec

### Problem

The middle column (SupervisorView) is an empty shell from Step 4. It needs to render tickets grouped by state as vertical swimlanes so a supervisor can see at a glance what needs their attention.

Currently there is no way to see supervisor-actionable tickets in the UI. The supervisor must use the CLI to identify what needs review, approval, or unblocking. The swimlane view gives a columnar overview of every ticket in a state that requires supervisor action, making the workscreen the primary interface for the supervision workflow.

The supervisor-actionable states (from config.toml `actionable = ["supervisor"]`) are: **question**, **specd**, **ammend**, **blocked**, **implemented**, and **accepted**. Swimlanes for states with no tickets must be hidden. Tickets within a swimlane are shown as compact summary cards. Clicking a card updates the global `selectedTicketId` in Zustand, which will drive the right-column detail panel (Step 6).

### Acceptance criteria

- [x] SupervisorView renders a horizontal row of swimlane columns, one per supervisor-actionable state that has at least one ticket
- [x] Swimlanes appear in a fixed order matching the workflow: question, specd, ammend, blocked, implemented, accepted
- [x] A swimlane with zero tickets is not rendered
- [x] Each swimlane has a header showing the state label and a count of tickets in that state
- [x] Each ticket is rendered as a card showing: short id (first 8 chars), title, agent name (or empty if unassigned), effort badge, risk badge
- [x] Clicking a ticket card sets selectedTicketId in the Zustand store to that ticket's id
- [x] The card for the currently selected ticket is visually highlighted
- [x] Ticket data is loaded from GET /api/tickets via TanStack Query
- [x] The swimlanes update automatically when the query refetches (no manual page reload required)

### Out of scope

- Keyboard arrow-key navigation across swimlanes (covered by Step 6)
- Ticket detail rendering (covered by Step 6)
- The worker activity panel and priority queue (covered by Step 7)
- State transition buttons on cards (covered by Step 8)
- Drag-and-drop reordering (covered by Step 11)
- Search or filter controls (covered by Step 14)
- Open question / amendment badges on cards (covered by Step 14c)

### Approach

**New files** (inside `apm-ui/src/`):

- `components/supervisor/SupervisorView.tsx` — top-level component placed inside the middle column panel from Step 4; renders the swimlane row
- `components/supervisor/Swimlane.tsx` — renders a single state column: header (label + count) and a scrollable list of TicketCard components
- `components/supervisor/TicketCard.tsx` — compact card for one ticket

**Data fetching**

Use the existing TanStack Query hook (established in Step 3) that calls `GET /api/tickets`. The response is an array of ticket objects with at minimum: `id`, `title`, `state`, `agent`, `effort`, `risk`.

**Supervisor-state filter**

Hard-code the ordered list of supervisor-actionable states:
```ts
const SUPERVISOR_STATES = ['question', 'specd', 'ammend', 'blocked', 'implemented', 'accepted'] as const;
```
These match the `actionable = ["supervisor"]` entries in `.apm/config.toml`. Hard-coding is intentional — the config is not served by the API at this stage. `ammend` is placed after `specd` to reflect the real workflow order (specd → ammend).

**Grouping logic** (inside SupervisorView)

1. Filter all tickets to only those whose `state` is in `SUPERVISOR_STATES`
2. Group into a `Map<state, Ticket[]>`
3. Iterate `SUPERVISOR_STATES` in order; skip any state with an empty group
4. Render one `Swimlane` per non-empty state

**TicketCard fields**

- Short id: `ticket.id.slice(0, 8)`
- Title: `ticket.title`
- Agent: `ticket.agent` or a dash if unassigned
- Effort badge: `E:{ticket.effort}` using shadcn Badge variant secondary; omit if effort is 0
- Risk badge: `R:{ticket.risk}` using shadcn Badge variant destructive for risk >= 7, secondary otherwise; omit if risk is 0
- Apply a ring highlight class when `ticket.id === selectedTicketId`

**Zustand wiring**

Read `selectedTicketId` and `setSelectedTicketId` from the Zustand store (established in Step 4). On card click, call `setSelectedTicketId(ticket.id)`.

**Styling**

Use shadcn Card for TicketCard and Tailwind for layout. The swimlane row uses `flex flex-row gap-4 overflow-x-auto h-full`. Each swimlane column uses `flex flex-col min-w-[220px] max-w-[280px]`.

### Open questions



### Amendment requests

- [x] Add `'ammend'` to the `SUPERVISOR_STATES` ordered list in the Approach — place it after `'specd'` (reflecting the real workflow order: specd → ammend). Tickets in `ammend` state require supervisor attention and must appear in the swimlanes.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:23Z | new | in_design | philippepascal |
| 2026-03-31T06:28Z | in_design | specd | claude-0331-0623-9a98 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:07Z | ammend | in_design | philippepascal |
| 2026-03-31T19:09Z | in_design | specd | claude-0331-1907-b4c2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-04-01T00:37Z | ready | in_progress | philippepascal |
| 2026-04-01T00:41Z | in_progress | implemented | claude-0401-0037-d1a8 |
| 2026-04-01T00:53Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |