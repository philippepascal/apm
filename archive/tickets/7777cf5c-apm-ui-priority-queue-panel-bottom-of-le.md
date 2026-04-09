+++
id = "7777cf5c"
title = "apm-ui: priority queue panel (bottom of left column, apm next ordering)"
state = "closed"
priority = 40
effort = 4
risk = 2
author = "apm"
agent = "23081"
branch = "ticket/7777cf5c-apm-ui-priority-queue-panel-bottom-of-le"
created_at = "2026-03-31T06:12:28.610477Z"
updated_at = "2026-04-01T04:55:02.514767Z"
+++

## Spec

### Problem

The bottom half of the `WorkerView` left column is a stub placeholder labelled "Queue" (put there by Step 7a, ticket 651f8a63). Supervisors currently have no browser-visible way to see which tickets are queued for dispatch or in what order the work engine will pick them up. They must leave the browser and run `apm next` from the CLI.

This ticket fills the placeholder with a `PriorityQueuePanel` showing all agent-actionable tickets ranked by the same scoring formula `apm next` uses: score = priority x priority_weight + effort x effort_weight + risk x risk_weight (weights from [workflow.prioritization] in apm.toml). The panel is read-only at this stage; drag-and-drop reordering is covered by ticket 7f61c54a (Step 11).

Two changes are required: (1) a GET /api/queue endpoint in apm-server returning all agent-actionable tickets sorted by descending score; (2) a PriorityQueuePanel React component wired into WorkerView.tsx replacing the stub placeholder.

### Acceptance criteria

- [x] `GET /api/queue` returns HTTP 200 with `Content-Type: application/json`
- [x] The response is a JSON array sorted by descending score using the formula `priority * priority_weight + effort * effort_weight + risk * risk_weight` with weights from `[workflow.prioritization]` in `apm.toml`
- [x] Each element in the response contains: `rank` (1-based integer), `id`, `title`, `state`, `priority`, `effort`, `risk`, `score`
- [x] Only tickets whose state is in `config.actionable_states_for("agent")` appear in the response
- [x] `GET /api/queue` returns an empty JSON array when no agent-actionable tickets exist
- [x] The handler offloads all blocking git work via `spawn_blocking` and does not block the tokio runtime
- [x] `PriorityQueuePanel` renders a row for each ticket in the response, showing rank, ID, title, state badge, effort, risk, and score
- [x] When the response array is empty, `PriorityQueuePanel` shows a centred "No tickets in queue." message
- [x] While the initial fetch is in-flight, `PriorityQueuePanel` shows loading skeleton rows
- [x] If the fetch fails, `PriorityQueuePanel` shows an inline error message
- [x] `PriorityQueuePanel` automatically refetches `GET /api/queue` every 10 seconds via TanStack Query `refetchInterval`
- [x] Clicking a queue row sets `selectedTicketId` in the Zustand store (the same global selection used by the swimlanes)
- [x] The row for the currently selected ticket is visually highlighted
- [x] `PriorityQueuePanel` is rendered in the bottom half of `WorkerView.tsx`, replacing the placeholder stub
- [x] `npm run build` in `apm-ui/` exits 0 with no TypeScript errors
- [x] `cargo test --workspace` passes

### Out of scope

- Drag-and-drop or keyboard reordering of the queue — covered by ticket 7f61c54a (Step 11)
- The top half of the left column (worker activity panel) — covered by ticket 651f8a63 (Step 7a)
- SSE/push-based live updates — polling every 10 seconds is sufficient at this stage
- Showing tickets in non-agent-actionable states (e.g. closed, specd, in_design)
- Keyboard navigation within the queue panel (arrow key focus, row-to-row) — deferred
- `DELETE /api/workers/:pid` or any worker control — Step 15 (ticket 6d46e15c)

### Approach

**Prerequisites:** Step 7a (ticket 651f8a63, worker activity panel) must be `implemented` before this ticket moves to `ready`. The `WorkerView.tsx` component and the bottom-half "Queue" stub it introduces are the integration points for this ticket.

---

### Step 0 — Update `Config::actionable_states_for` return type (apm-core)

`apm-core/src/config.rs` already defines `Config::actionable_states_for(&self, actor: &str) -> Vec<&'_ str>`. The queue handler calls this inside a `spawn_blocking` closure, which requires all captured values to be `'static`; a `Vec<&'_ str>` tied to `Config`'s lifetime cannot cross that boundary.

Change the return type to `Vec<String>` and update the body to `.map(|s| s.id.clone()).collect()`. Update any callers in `apm-core` (e.g. `start.rs`, `ticket.rs`) that currently compare against `&str` slices — usually a trivial `.as_str()` call. Add a unit test confirming `actionable_states_for("agent")` returns a vec containing `"ready".to_string()`.

Do this before implementing the server handler.

---

### apm-core: expose sorted queue helper

Add `pub fn sorted_actionable<'a>(tickets: &'a [Ticket], actionable: &[&str], pw: f64, ew: f64, rw: f64) -> Vec<&'a Ticket>` to `apm-core/src/ticket.rs` alongside `pick_next`. It filters by actionable states, sorts descending by score, and returns the full ranked slice. Refactor `pick_next` to call `sorted_actionable(...).into_iter().next()` so the scoring formula lives in one place.

This avoids duplicating the sort logic in the server handler and makes the formula testable independently.

---

### apm-server: `GET /api/queue`

1. Add `apm-server/src/routes/queue.rs` (~55 lines):

```rust
#[derive(serde::Serialize)]
struct QueueEntry {
    rank: usize,
    id: String,
    title: String,
    state: String,
    priority: u8,
    effort: u8,
    risk: u8,
    score: f64,
}
```

2. `queue_handler(State(app): State<AppState>) -> axum::Json<Vec<QueueEntry>>`:
   - Runs inside `tokio::task::spawn_blocking`
   - Loads config via `Config::load(&root)`
   - Calls `ticket::load_all_from_git(&root, &config.tickets.dir)`
   - Gets actionable states: `config.actionable_states_for("agent")`
   - Gets prioritization weights: `&config.workflow.prioritization`
   - Calls `ticket::sorted_actionable(&tickets, &actionable, pw, ew, rw)`
   - Maps to `Vec<QueueEntry>` with 1-based rank, `score` rounded to 2 decimal places
   - Returns `axum::Json(entries)`

3. Register in `apm-server/src/main.rs`:
```rust
.route("/api/queue", get(queue::queue_handler))
```
Add `mod queue;` import.

---

### apm-ui: PriorityQueuePanel

1. Add `apm-ui/src/components/PriorityQueuePanel.tsx` (~90 lines):

```ts
interface QueueEntry {
  rank: number;
  id: string;
  title: string;
  state: string;
  priority: number;
  effort: number;
  risk: number;
  score: number;
}
```

- `useQuery({ queryKey: ['queue'], queryFn: () => fetch('/api/queue').then(r => r.json()), refetchInterval: 10_000 })`
- Read `selectedTicketId` and `setSelectedTicketId` from Zustand store
- **Loading:** render 3 `<Skeleton>` rows (shadcn Skeleton)
- **Error:** render short error card with message
- **Empty:** render centred `<p>No tickets in queue.</p>`
- **Populated:** render a scrollable shadcn `Table` with columns: **#**, **ID**, **Title**, **State**, **E**, **R**, **Score**
  - **#** column: 1-based rank number
  - **ID** column: first 8 chars of ticket id
  - **State** column: `<Badge variant="outline">` with state label
  - **E / R** columns: effort / risk numeric values
  - **Score** column: score to 1 decimal place
  - Row `onClick`: `setSelectedTicketId(entry.id)`
  - Row highlighted (e.g. `bg-accent`) when `entry.id === selectedTicketId`

2. Integrate into `apm-ui/src/components/WorkerView.tsx`:
   - Import `PriorityQueuePanel`
   - Replace the "Queue" placeholder stub with `<PriorityQueuePanel />`
   - The existing `<Separator />` between top and bottom halves (from Step 7a) stays in place

---

### File changes summary

| File | Change |
|------|--------|
| `apm-core/src/ticket.rs` | Add `sorted_actionable`; refactor `pick_next` to delegate |
| `apm-server/src/routes/queue.rs` | New file — queue handler |
| `apm-server/src/main.rs` | Add `mod queue` and `.route("/api/queue", ...)` |
| `apm-ui/src/components/PriorityQueuePanel.tsx` | New file — queue panel component |
| `apm-ui/src/components/WorkerView.tsx` | Replace placeholder with `<PriorityQueuePanel />` |

---

### Order of steps

1. Add `sorted_actionable` to `apm-core` and update `pick_next` — run `cargo test --workspace` to confirm no regression
2. Add `queue.rs` route and register it in `main.rs`
3. Add `PriorityQueuePanel.tsx`
4. Wire it into `WorkerView.tsx`
5. Run `npm run build` in `apm-ui/` and `cargo test --workspace`

### Open questions



### Amendment requests

- [x] Add `config.actionable_states_for(actor: &str) -> Vec<String>` to apm-core Config (scan `[[workflow.states]]` for entries whose `actionable` array contains the given actor string). Both the queue handler and the dry-run handler depend on this method — it must be defined in apm-core before either handler is implemented. Include this step at the top of the Approach.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:38Z | new | in_design | philippepascal |
| 2026-03-31T06:42Z | in_design | specd | claude-0331-0638-c698 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:07Z | ammend | in_design | philippepascal |
| 2026-03-31T19:11Z | in_design | specd | claude-0331-1907-b2f1 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T02:13Z | ready | in_progress | philippepascal |
| 2026-04-01T02:31Z | in_progress | implemented | claude-0401-0213-c3d0 |
| 2026-04-01T02:34Z | implemented | accepted | apm-sync |
| 2026-04-01T04:55Z | accepted | closed | apm-sync |