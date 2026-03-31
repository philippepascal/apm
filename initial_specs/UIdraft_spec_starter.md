UI draft spec starter
1. layout start with a stack, even though for now only one screen, called workscreen. at some point we'll probably add a login
2. workscreen has 3 columns. all resizable/hidable, always at least on is visible
3. left most column is workerview
4. middle column is supervisorview
5. right column is ticket details
6. workerview is split in top: current activity, showing the tickets worker are working on, their state, etc
7. workview bottom shows the queue of actionable tickets in priority order (based on same algo that apm next uses)
8. middle column shows vertical swimlanes of tickets. each swimlane is based on the state machine. only supervisor actionable states are shown. only non empty columns are shown. tickets are summarize, show their states, etc
9. right column is ticket detail of the currently selected ticket
10. ticket selection is global across workerview and supervisor view
11. tickets can be navigate with arrows, including across workerview and supervisor view
12. ticket detail is a markdown viewer. RO. 
13. ticket detail, has button for review which opens an editor screen that takes the space of supervisorview and ticketdetail. ideally workerview is still visible.
14. editor screen is markdown editor, with RO sections (where user shouldn't change things, based on the config.toml configuation) and RW sections where he should be able to change things. it maintain correct format, either through UI tricks (checkboxes), or auto formatting.
15. editor screen has buttons to move ticket to possible states, including close or keep_at_<currentstate>, with same behavior as the command line
16. all operation have a keyboard shortcut
17. user can start/stop the apm work engine with a button on top of the workerview or keyboard shortcut
18. users can move ticket in workerview queue, which automatically adjust their priority
19. 




Stack highlevel
rust axum/tokio
React/node.js
Vite
TanStack Query
Zustand
shadcn/ui
MD editor CodeMirror 6

inspired by linear


---

## Implementation plan

Each step produces a working, shippable slice. Validate the technology choice before the next step depends on it.

---

### Step 1 — Rust HTTP server skeleton
- Add `apm-server` crate 
- Wire in `axum` + `tokio`; single `GET /health` endpoint returns `{"ok":true}`
- No business logic yet — goal is to confirm the crate compiles, ships, and serves

### Step 2 — Ticket list API
- `GET /api/tickets` — calls `ticket::load_all_from_git()` from `apm-core`, returns JSON array
- `GET /api/tickets/:id` — returns single ticket as JSON (frontmatter + body)
- Validates that `apm-core` logic is callable from an async context with no blocking issues

### Step 3 — React + Vite skeleton
- `apm-ui/` directory: Vite + React + TypeScript + shadcn/ui
- Single blank page, served as static files from the axum server (`GET /` → index.html)
- TanStack Query installed; one `useQuery` call to `/api/tickets` logs results to console
- Goal: confirm full stack wires together end-to-end before building any UI

### Step 4 — 3-column layout shell
- Implement the resizable/hidable 3-column layout with empty panels
- Zustand store: `selectedTicketId`, column visibility flags
- No data rendered yet — validate layout behaviour (resize, hide, keyboard focus between columns)

### Step 5 — Supervisor swimlanes (middle column)
- Render tickets grouped by state as vertical swimlanes
- Only show supervisor-actionable states; hide empty columns
- Tickets shown as summary cards (id, title, agent, effort/risk badges)
- Ticket click → sets `selectedTicketId` in Zustand

### Step 6 — Ticket detail panel (right column)
- `GET /api/tickets/:id` → render full ticket markdown as read-only (remark or similar)
- Updates reactively when `selectedTicketId` changes
- Keyboard navigation: arrow keys move selection across swimlanes and worker queue

### Step 7 — Worker activity panel (left column, top half)
- `GET /api/workers` — lists running worker processes and which ticket each holds
- Polled on a short interval (or SSE); shows ticket id, state, agent name
- Left column bottom: render the `apm next` priority queue (same ordering algorithm)

### Step 8 — State transition API + buttons
- `POST /api/tickets/:id/transition` `{ "to": "<state>" }` — calls the state machine in `apm-core`
- Editor screen: render valid transitions as buttons; wire to API
- "Close" and "keep at current state" included
- Validate that state machine errors surface cleanly in the UI

### Step 9 — Markdown editor with RO/RW sections (CodeMirror 6)
- Replace read-only detail view with CodeMirror 6 editor in review mode
- Mark frontmatter and `## History` ranges as read-only using CodeMirror compartments
- RW sections: free text editing; checkboxes rendered as interactive UI elements
- `PUT /api/tickets/:id/body` — saves edited body back to the ticket branch via git

### Step 10 — apm new form
- "+ New ticket" button / keyboard shortcut opens a modal
- Fields: title (required), problem, acceptance criteria, approach, out-of-scope (all optional)
- `POST /api/tickets` — calls `ticket::create()` in `apm-core`
- Spec sections written atomically at creation (parity with the `--section/--set` CLI feature)

### Step 11 — Priority reorder in worker queue
- Drag-and-drop (or up/down keyboard shortcuts) on the queue in the left column
- `PATCH /api/tickets/:id` `{ "priority": N }` — persists to ticket branch
- Reorder updates immediately in the queue; swimlanes unaffected

### Step 12 — apm work engine controls
- Start/stop button at top of workerview → `POST /api/work/start` / `POST /api/work/stop`
- Status indicator (running / stopped / idle)
- Dry-run preview: "what would be dispatched" before starting

### Step 13 — Sync + metadata editing
- "Sync" button → `POST /api/sync` — runs `apm sync` logic, refreshes all ticket data
- Inline `effort`, `risk`, `priority` editors on the detail panel (click-to-edit)
- `PATCH /api/tickets/:id` for field updates

### Step 14 — Search, filter, and observability
- Text search across ticket titles and bodies (client-side filter on cached data)
- Filter by agent, state, or "show closed"
- Log tail panel: `GET /api/log/stream` (SSE) → scrolling log viewer
- Visual badges for tickets with open questions or pending amendment requests

### Step 15 — Worker management
- `GET /api/workers` extended: show PID, uptime, ticket branch
- Stop individual worker: `DELETE /api/workers/:pid`
- `apm take` equivalent: reassign ticket to current user from the detail panel

---

Each step can be delivered as its own PR. Steps 1–3 are the integration proof — if anything in the stack is wrong, it shows up there before the UI is built on top of it.
