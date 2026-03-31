+++
id = "268f5694"
title = "apm-ui: ticket detail panel with markdown viewer and keyboard navigation"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "20759"
branch = "ticket/268f5694-apm-ui-ticket-detail-panel-with-markdown"
created_at = "2026-03-31T06:12:10.547637Z"
updated_at = "2026-03-31T06:28:34.107396Z"
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


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:28Z | new | in_design | philippepascal |