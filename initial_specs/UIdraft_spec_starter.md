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

---

## Ticket creation commands

Run these in order. Each command pre-populates the Problem section so spec writers have full context immediately. The step number in the title maps directly to the implementation plan above. Prerequisites must be `implemented` before the dependent ticket moves to `ready`.

```bash
# Step 1 — no prerequisites
apm new --no-edit "apm-server: axum/tokio skeleton with GET /health endpoint" \
  --section Problem --set "The UI needs a Rust HTTP backend. Create the apm-server crate (or apm serve command) with axum + tokio. The only endpoint at this stage is GET /health returning {\"ok\":true}. No business logic yet — the goal is to confirm the crate compiles, ships, and serves. Full spec context: initial_specs/UIdraft_spec_starter.md Step 1."

# Step 2 — requires Step 1 implemented
apm new --no-edit "apm-server: ticket list and detail API endpoints" \
  --section Problem --set "The frontend needs read access to tickets. Add GET /api/tickets (all tickets as JSON array via ticket::load_all_from_git) and GET /api/tickets/:id (single ticket, frontmatter + body). This also validates that apm-core logic works correctly in an async axum context. Full spec context: initial_specs/UIdraft_spec_starter.md Step 2. Requires Step 1."

# Step 3 — requires Step 2 implemented
apm new --no-edit "apm-ui: Vite + React + shadcn/ui skeleton wired to backend" \
  --section Problem --set "There is no frontend yet. Create the apm-ui/ directory with Vite + React + TypeScript + shadcn/ui. The page is blank but TanStack Query is installed and one useQuery call to /api/tickets logs results to console. Static files are served by the axum server at GET /. Goal: confirm the full stack wires together end-to-end before any UI is built on top. Full spec context: initial_specs/UIdraft_spec_starter.md Step 3. Requires Step 2."

# Step 4 — requires Step 3 implemented
apm new --no-edit "apm-ui: 3-column resizable/hidable layout shell with Zustand" \
  --section Problem --set "The workscreen layout (3 resizable/hidable columns: workerview, supervisorview, ticket detail) needs to be established before any data is rendered into it. Zustand store holds selectedTicketId and column visibility flags. No data rendered yet — validate resize, hide, and keyboard focus between columns. Full spec context: initial_specs/UIdraft_spec_starter.md Step 4. Requires Step 3."

# Step 5 — requires Step 4 implemented
apm new --no-edit "apm-ui: supervisor swimlanes in middle column" \
  --section Problem --set "The middle column shows tickets grouped by state as vertical swimlanes. Only supervisor-actionable states are shown; empty columns are hidden. Tickets appear as summary cards with id, title, agent, effort/risk badges. Clicking a card sets selectedTicketId in Zustand. Full spec context: initial_specs/UIdraft_spec_starter.md Step 5. Requires Step 4."

# Step 6 — requires Step 5 implemented
apm new --no-edit "apm-ui: ticket detail panel with markdown viewer and keyboard navigation" \
  --section Problem --set "The right column shows the full ticket content as a read-only markdown view, updating reactively when selectedTicketId changes. Arrow key navigation moves selection across swimlanes and the worker queue globally. Full spec context: initial_specs/UIdraft_spec_starter.md Step 6. Requires Step 5."

# Step 7a — requires Step 6 implemented
apm new --no-edit "apm-server + apm-ui: worker activity panel (running workers, top of left column)" \
  --section Problem --set "The top half of the left column shows running worker processes and which ticket each holds. Add GET /api/workers listing PID, agent name, ticket id, and state. The panel polls on a short interval or uses SSE. Full spec context: initial_specs/UIdraft_spec_starter.md Step 7. Requires Step 6."

# Step 7b — requires Step 7a implemented
apm new --no-edit "apm-ui: priority queue panel (bottom of left column, apm next ordering)" \
  --section Problem --set "The bottom half of the left column shows the queue of actionable tickets in the same priority order as apm next. This is read-only at this stage; reordering is covered by a later ticket. Full spec context: initial_specs/UIdraft_spec_starter.md Step 7. Requires Step 7a."

# Step 8 — requires Step 6 implemented
apm new --no-edit "apm-server + apm-ui: state transition API and buttons" \
  --section Problem --set "There is no way to transition ticket state from the UI. Add POST /api/tickets/:id/transition {\"to\":\"<state>\"} backed by the apm-core state machine. The ticket detail panel gains buttons for all valid transitions from the current state, including close and keep-at-current-state, matching CLI behaviour. Full spec context: initial_specs/UIdraft_spec_starter.md Step 8. Requires Step 6."

# Step 9 — requires Step 8 implemented
apm new --no-edit "apm-ui: markdown editor with RO/RW sections (CodeMirror 6) and save API" \
  --section Problem --set "The review button on the ticket detail panel should open a full markdown editor. Frontmatter and the History section must be read-only (CodeMirror compartments); all other sections are editable. Checkboxes render as interactive UI elements. Add PUT /api/tickets/:id/body to commit the edited content back to the ticket branch. Full spec context: initial_specs/UIdraft_spec_starter.md Step 9. Requires Step 8."

# Step 10 — requires Step 9 implemented
apm new --no-edit "apm-server + apm-ui: new ticket form with section pre-population" \
  --section Problem --set "There is no way to create a ticket from the UI. A '+ New ticket' button/shortcut opens a modal with fields for title (required) and optional spec sections (problem, acceptance criteria, out of scope, approach). Add POST /api/tickets backed by ticket::create in apm-core. Sections are written atomically at creation. Full spec context: initial_specs/UIdraft_spec_starter.md Step 10. Requires Step 9."

# Step 11 — requires Step 7b implemented
apm new --no-edit "apm-ui: priority reorder via drag-and-drop in worker queue" \
  --section Problem --set "The priority queue in the left column is currently read-only. Users need to reorder tickets to influence what apm next dispatches next. Add drag-and-drop (and up/down keyboard shortcuts) that call PATCH /api/tickets/:id {\"priority\":N} to persist the new order. Full spec context: initial_specs/UIdraft_spec_starter.md Step 11. Requires Step 7b."

# Step 12a — requires Step 7a implemented
apm new --no-edit "apm-server + apm-ui: apm work engine start/stop controls" \
  --section Problem --set "There is no way to start or stop the apm work daemon from the UI. Add POST /api/work/start and POST /api/work/stop endpoints. The top of the workerview panel shows a start/stop button with a status indicator (running / stopped / idle) and a keyboard shortcut. Full spec context: initial_specs/UIdraft_spec_starter.md Step 12. Requires Step 7a."

# Step 12b — requires Step 12a implemented
apm new --no-edit "apm-ui: apm work dry-run preview before engine start" \
  --section Problem --set "Users need to see what would be dispatched before starting the work engine. Add a dry-run preview panel (backed by GET /api/work/dry-run) that shows candidate tickets and their intended workers, visible before clicking start. Full spec context: initial_specs/UIdraft_spec_starter.md Step 12. Requires Step 12a."

# Step 13a — requires Step 4 implemented
apm new --no-edit "apm-server + apm-ui: sync button (POST /api/sync)" \
  --section Problem --set "The UI has no way to pull the latest ticket state from git branches. Add POST /api/sync that runs apm sync logic and refreshes all ticket data. A sync button in the UI (with keyboard shortcut) triggers this and shows a loading state while in progress. Full spec context: initial_specs/UIdraft_spec_starter.md Step 13. Requires Step 4."

# Step 13b — requires Step 9 implemented
apm new --no-edit "apm-ui: inline effort/risk/priority editing in ticket detail" \
  --section Problem --set "effort, risk, and priority fields in the ticket detail panel are read-only. Users need click-to-edit inline controls for these fields, backed by PATCH /api/tickets/:id, without opening the full markdown editor. Full spec context: initial_specs/UIdraft_spec_starter.md Step 13. Requires Step 9."

# Step 14a — requires Step 5 implemented
apm new --no-edit "apm-ui: ticket search and filter (by state, agent, text)" \
  --section Problem --set "There is no way to filter the ticket list beyond the swimlane grouping. Add client-side text search across titles and bodies, and filter controls for state, agent, and show-closed toggle (parity with apm list --state and --all). Full spec context: initial_specs/UIdraft_spec_starter.md Step 14. Requires Step 5."

# Step 14b — requires Step 12a implemented
apm new --no-edit "apm-server + apm-ui: log tail viewer via SSE" \
  --section Problem --set "There is no visibility into the apm log from the UI. Add GET /api/log/stream as a Server-Sent Events endpoint tailing the configured log file. A collapsible log panel in the UI shows the live stream. Full spec context: initial_specs/UIdraft_spec_starter.md Step 14. Requires Step 12a."

# Step 14c — requires Step 5 implemented
apm new --no-edit "apm-ui: open question and amendment request badges on ticket cards" \
  --section Problem --set "Ticket summary cards in the swimlanes give no indication of whether a ticket has open questions or pending amendment requests. Add visual badges derived from the ticket body so supervisors can triage at a glance without opening the detail panel. Full spec context: initial_specs/UIdraft_spec_starter.md Step 14. Requires Step 5."

# Step 15 — requires Step 7a and Step 8 implemented
apm new --no-edit "apm-server + apm-ui: worker management (list, stop, reassign)" \
  --section Problem --set "The worker activity panel shows running workers but provides no controls. Extend GET /api/workers with PID and uptime, add DELETE /api/workers/:pid to stop a worker, and add a reassign action (apm take equivalent) on the ticket detail panel. Full spec context: initial_specs/UIdraft_spec_starter.md Step 15. Requires Step 7a and Step 8."
```

---

## Keyboard shortcuts

All operations must have a keyboard shortcut (point 16). Shortcuts are global unless noted as context-specific.

### Navigation

| Key | Action | Notes |
|-----|--------|-------|
| `↑` / `↓` | Move selection within current column | Focus must be on a ticket card, not in text |
| `←` / `→` | Move focus between columns (WorkerView ↔ SupervisorView ↔ TicketDetail) | Only when focus is on a card |
| `Enter` | Open selected ticket in detail panel | |
| `Escape` | Close editor / dismiss modal / return focus to card grid | |

### Ticket operations (selected ticket)

| Key | Action | CLI equivalent |
|-----|--------|----------------|
| `n` | New ticket (open modal) | `apm new` |
| `r` | Review selected ticket (open editor) | `apm review` |
| `s` | Show raw ticket in detail panel | `apm show` |
| `t` | Take selected ticket (reassign to self) | `apm take` |
| `Shift+S` | Sync with remote | `apm sync` |
| `Shift+C` | Close selected ticket (supervisor only) | `apm close` |

### State transitions (selected ticket, context: detail panel or editor)

### Work engine

| Key | Action | CLI equivalent |
|-----|--------|----------------|
| `Shift+W` | Start / stop `apm work` daemon | `apm work` / stop |

### Workers

| Key | Action | CLI equivalent |
|-----|--------|----------------|
| `Shift+K` | Stop selected worker | `apm workers stop` |

### Column visibility

Column visibility is toggled via toolbar buttons only — no keyboard shortcut. `Ctrl+1/2/3` conflicts with browser tab switching on Windows/Linux (Chrome, Firefox, Edge) and cannot be reliably used. `Alt+1/2/3` conflicts with system menu navigation on Windows. Toolbar buttons are the safe, cross-platform solution.

### State transitions Editor (when markdown editor is open)
Transitions only happen in review screen (with the exception of close).
Only valid transitions are shown and are based on config. 
Shortcut letter is calculated using an algorithm based on state name and avoid conflict.

| Key | Action |
|-----|--------|
| `Escape` | Discard changes and close editor |

### Notes
- `1`–`9` is NOT used directly for effort/risk — it would fire while typing in search or other inputs. Instead, `e` and `Shift+R` open a focused picker that captures the next digit keystroke, then closes.
- Arrow key navigation between columns should only fire when focus is on a ticket card, not when focus is inside a text input or the markdown editor, to avoid conflicting with normal text navigation.
- All shortcuts should be discoverable via a `?` help overlay.
