+++
id = "651f8a63"
title = "apm-server + apm-ui: worker activity panel (running workers, top of left column)"
state = "closed"
priority = 45
effort = 4
risk = 3
author = "apm"
agent = "53860"
branch = "ticket/651f8a63-apm-server-apm-ui-worker-activity-panel-"
created_at = "2026-03-31T06:12:27.354130Z"
updated_at = "2026-04-01T04:54:58.099240Z"
+++

## Spec

### Problem

The top half of the left column (WorkerView) in `apm-ui` currently shows a placeholder stub from Step 4. There is no way to see which worker processes are running or which tickets they hold without leaving the browser and using the CLI. This creates an observability gap: supervisors must context-switch to the terminal to assess worker health.

Adding `GET /api/workers` to `apm-server` and wiring up a WorkerActivityPanel component in `apm-ui` gives supervisors an at-a-glance view of running workers, their assigned tickets, current state, agent name, and elapsed time — all without leaving the browser.

**Current state:** Left column top half is a stub/placeholder (from Step 4). `apm-core/src/worker.rs` already implements `read_pid_file`, `is_alive`, and `elapsed_since` — tested and ready to call. Workers write `.apm-worker.pid` files to their worktrees (implemented by ticket 0084).

**Desired state:** The WorkerActivityPanel polls `GET /api/workers` every 5 seconds and renders a list of live and crashed workers with their ticket id, title, state, agent name, and elapsed time.

### Acceptance criteria

- [x] `GET /api/workers` returns HTTP 200 with `Content-Type: application/json`
- [x] The response body is a JSON array; each element contains: `pid`, `ticket_id`, `ticket_title`, `state`, `agent`, `elapsed`, `status` (`"running"` or `"crashed"`)
- [x] `GET /api/workers` returns an empty JSON array when no `.apm-worker.pid` files exist in any worktree
- [x] A worker whose PID is no longer alive appears in the response with `status: "crashed"` (not silently omitted)
- [x] The handler offloads all blocking work via `spawn_blocking` and does not block the tokio runtime
- [x] WorkerActivityPanel renders a table row for each worker in the response
- [x] Each row shows: ticket ID, ticket title, agent name, current state, elapsed time, and a status badge
- [x] When the array is empty, WorkerActivityPanel shows a centred "No workers running." message
- [x] WorkerActivityPanel polls `GET /api/workers` automatically every 5 seconds via TanStack Query `refetchInterval`
- [x] While the initial fetch is in-flight, WorkerActivityPanel shows a loading skeleton
- [x] If the fetch returns an error, WorkerActivityPanel shows an error message
- [x] `npm run build` in `apm-ui/` exits 0 with no TypeScript errors
- [x] `cargo test --workspace` passes

### Out of scope

- Stop or kill a worker from the UI — covered by ticket 6d46e15c (Step 15: worker management)
- Extending the response with additional fields (uptime, branch) beyond what is listed above — Step 15
- The priority queue in the bottom half of the left column — covered by ticket 7777cf5c (Step 7b)
- SSE-based push updates — polling at 5-second intervals is sufficient
- Keyboard navigation into the worker panel — noted as out-of-scope in Step 6; deferred to Step 7 extension
- `DELETE /api/workers/:pid` endpoint — Step 15

### Approach

**Prerequisites:** Step 6 (ticket 268f5694, ticket detail panel) must be `implemented` before this ticket moves to `ready`.

---

**apm-server: `GET /api/workers`**

1. Add `apm-server/src/routes/workers.rs` (new file, ~60 lines):

   ```rust
   #[derive(serde::Serialize)]
   struct WorkerInfo {
       pid: u32,
       ticket_id: String,
       ticket_title: String,
       state: String,
       agent: String,
       elapsed: String,
       status: String,  // "running" | "crashed"
   }
   ```

2. `workers_handler` runs inside `tokio::task::spawn_blocking`:
   - Run `git worktree list --porcelain` (via `std::process::Command`) in the repo root
   - Parse stdout lines beginning with `"worktree "` to collect worktree paths
   - For each path: check for `<path>/.apm-worker.pid`; skip if absent
   - Call `apm_core::worker::read_pid_file(&pid_path)` → `(pid, PidFile)`
   - `apm_core::worker::is_alive(pid)` → status string `"running"` or `"crashed"`
   - `apm_core::worker::elapsed_since(&pf.started_at)` → elapsed string
   - Load all tickets with `apm_core::ticket::load_all_from_git(&root, &tickets_dir)` (once, outside the per-worktree loop)
   - Find the matching ticket by `frontmatter.id == pf.ticket_id`; fall back to empty strings if not found
   - Collect results into `Vec<WorkerInfo>`; return `axum::Json(results)`

3. Register in `apm-server/src/main.rs`:
   ```rust
   .route("/api/workers", get(workers::workers_handler))
   ```

**apm-ui: WorkerActivityPanel**

1. Add `apm-ui/src/components/WorkerActivityPanel.tsx` (~80 lines):

   ```ts
   interface WorkerInfo {
     pid: number;
     ticket_id: string;
     ticket_title: string;
     state: string;
     agent: string;
     elapsed: string;
     status: 'running' | 'crashed';
   }
   ```

   - `useQuery({ queryKey: ['workers'], queryFn: () => fetch('/api/workers').then(r => r.json()), refetchInterval: 5000 })`
   - Loading → render `<Skeleton>` rows (shadcn Skeleton, ~3 rows)
   - Error → render a short error card
   - Empty array → render centred "No workers running." text
   - Populated → render a shadcn `Table` with columns: **ID**, **Title**, **Agent**, **State**, **Elapsed**, **Status**
   - Status badge: "running" → green `<Badge variant="outline">`; "crashed" → red `<Badge variant="destructive">`

2. Integrate into `apm-ui/src/components/WorkerView.tsx`:
   - Import and render `<WorkerActivityPanel />` in the top section of the left column
   - Add a visual separator (`<Separator />` from shadcn) between top and bottom halves
   - Bottom half remains a placeholder labelled "Queue" (filled by Step 7b)

**File changes summary:**
- `apm-server/src/routes/workers.rs` — new file
- `apm-server/src/main.rs` — register route; add `mod workers` import
- `apm-ui/src/components/WorkerActivityPanel.tsx` — new file
- `apm-ui/src/components/WorkerView.tsx` — integrate panel and separator

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:33Z | new | in_design | philippepascal |
| 2026-03-31T06:38Z | in_design | specd | claude-0330-2045-spec1 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T01:23Z | ready | in_progress | philippepascal |
| 2026-04-01T01:30Z | in_progress | implemented | claude-0401-0123-3778 |
| 2026-04-01T01:36Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |