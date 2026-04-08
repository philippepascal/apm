+++
id = "6d46e15c"
title = "apm-server + apm-ui: worker management (list, stop, reassign)"
state = "closed"
priority = 30
effort = 5
risk = 3
author = "apm"
agent = "78853"
branch = "ticket/6d46e15c-apm-server-apm-ui-worker-management-list"
created_at = "2026-03-31T06:13:21.657306Z"
updated_at = "2026-04-01T07:12:44.596656Z"
+++

## Spec

### Problem

The worker activity panel (added in Step 7a, ticket 651f8a63) is read-only: supervisors can see which workers are running but cannot stop a misbehaving worker or reassign a stalled ticket from the browser. Two controls are missing:

1. Stop worker - no way to SIGTERM an individual worker process from the UI. The CLI has apm workers --kill but the HTTP API has no DELETE endpoint.
2. Reassign ticket - no way to call the equivalent of apm take from the UI. A ticket stuck in_progress under a crashed or gone worker requires CLI access to reassign.

Additionally, the GET /api/workers response (specced in Step 7a) does not include the ticket branch, which is useful context when stopping a worker or deciding whether to reassign.

Current state (after Steps 7a and 8): WorkerActivityPanel shows a table of live/crashed workers with pid, ticket_id, title, state, agent, elapsed, and status. TicketDetail shows transition buttons. Neither has stop or reassign controls.

Desired state: WorkerActivityPanel gains a Stop button per row (live workers only). TicketDetail gains a Reassign to me button. The backend gains DELETE /api/workers/:pid and POST /api/tickets/:id/take.

### Acceptance criteria

- [x] GET /api/workers response includes a `branch` field (the ticket's branch name) for each worker entry
- [x] DELETE /api/workers/:pid returns 204 when the given PID is found in a worktree pid file, is alive, and is successfully sent SIGTERM
- [x] DELETE /api/workers/:pid removes the `.apm-worker.pid` file from the worktree after sending SIGTERM
- [x] DELETE /api/workers/:pid returns 404 when no `.apm-worker.pid` file in any worktree contains the given PID
- [x] DELETE /api/workers/:pid returns 409 when the PID is found in a pid file but the process is no longer alive (stale pid file)
- [x] POST /api/tickets/:id/take reassigns the ticket's agent field to the server's resolved agent name (APM_AGENT_NAME env var, falling back to USER) and returns 200 with the updated ticket JSON
- [x] POST /api/tickets/:id/take returns 404 when the ticket id does not exist
- [x] WorkerActivityPanel shows a "Stop" button for each worker row where status is "running"
- [x] Clicking "Stop" in WorkerActivityPanel calls DELETE /api/workers/:pid; on success the worker list refreshes and the row disappears or shows "crashed"
- [x] Clicking "Stop" disables the button while the DELETE request is in-flight
- [x] On a DELETE failure, WorkerActivityPanel shows an inline error message near the row
- [x] Pressing Shift+K while a worker row is focused triggers the same Stop action as clicking the Stop button
- [x] TicketDetail shows a "Reassign to me" button when a ticket is selected
- [x] Clicking "Reassign to me" calls POST /api/tickets/:id/take and, on success, updates the agent badge in the detail panel
- [x] On a take failure, TicketDetail shows an inline error message near the button
- [x] npm run build in apm-ui/ exits 0 with no TypeScript errors
- [x] cargo test --workspace passes

### Out of scope

- Killing a worker by ticket id from the API (the CLI uses ticket id, but the HTTP endpoint uses PID directly — no ticket id → pid lookup needed in this ticket)
- Sending signals other than SIGTERM (SIGKILL fallback, graceful drain) — not needed at this scale
- The log tail viewer — covered by ticket e9ba2503 (Step 14)
- Authentication or authorisation for the stop/take actions — no auth layer exists yet
- The priority queue in the bottom half of the left column — covered by ticket 7777cf5c (Step 7b)
- Streaming or SSE updates for the worker list — polling every 5 seconds (established in Step 7a) is sufficient
- Reassigning a ticket to a specific named agent other than the server's own identity — the UI has no agent picker at this step

### Approach

**Prerequisites:** Step 7a (ticket 651f8a63) and Step 8 (ticket 8c7d47f0) must be `implemented` before this ticket moves to `ready`. This ticket builds directly on the `WorkerInfo` struct, `WorkerActivityPanel`, `TicketDetail`, and `AppState` established in those steps.

---

**1. Extend GET /api/workers — add `branch` field**

In `apm-server/src/routes/workers.rs`, add `branch: String` to the `WorkerInfo` struct:

```rust
#[derive(serde::Serialize)]
struct WorkerInfo {
    pid: u32,
    ticket_id: String,
    ticket_title: String,
    branch: String,   // new
    state: String,
    agent: String,
    elapsed: String,
    status: String,
}
```

In the handler loop, after finding the matching ticket, set:
```rust
branch: t.map(|t| {
    t.frontmatter.branch.clone()
        .or_else(|| apm_core::git::branch_name_from_path(&t.path))
        .unwrap_or_default()
}).unwrap_or_default(),
```

---

**2. Add DELETE /api/workers/:pid**

In `apm-server/src/routes/workers.rs`, add a handler `delete_worker`:

```rust
async fn delete_worker(
    State(state): State<Arc<AppState>>,
    Path(pid_str): Path<String>,
) -> impl IntoResponse {
    let pid: u32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, ...).into_response(),
    };
    let root = state.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        stop_worker_by_pid(&root, pid)
    }).await.unwrap();
    match result {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(StopError::NotFound) => (StatusCode::NOT_FOUND, Json(json!({"error":"pid not found"}))).into_response(),
        Err(StopError::NotAlive) => (StatusCode::CONFLICT, Json(json!({"error":"process not alive (stale pid file)"}))).into_response(),
        Err(StopError::Other(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    }
}

enum StopError { NotFound, NotAlive, Other(String) }

fn stop_worker_by_pid(root: &Path, target_pid: u32) -> Result<(), StopError> {
    let worktrees = apm_core::git::list_ticket_worktrees(root)
        .map_err(|e| StopError::Other(e.to_string()))?;
    for (wt_path, _branch) in &worktrees {
        let pid_path = wt_path.join(".apm-worker.pid");
        if !pid_path.exists() { continue; }
        let Ok((pid, _)) = apm_core::worker::read_pid_file(&pid_path) else { continue; };
        if pid != target_pid { continue; }
        if !apm_core::worker::is_alive(pid) {
            let _ = std::fs::remove_file(&pid_path);
            return Err(StopError::NotAlive);
        }
        std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| StopError::Other(e.to_string()))?;
        let _ = std::fs::remove_file(&pid_path);
        return Ok(());
    }
    Err(StopError::NotFound)
}
```

Register in `apm-server/src/main.rs`:
```rust
.route("/api/workers/:pid", delete(workers::delete_worker))
```

---

**3. Add POST /api/tickets/:id/take**

In `apm-server/src/routes/tickets.rs`, add a `take_ticket` handler:

```rust
async fn take_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let root = state.root.clone();
    let config = state.config.clone();
    let result = tokio::task::spawn_blocking(move || {
        let agent_name = apm_core::start::resolve_agent_name();
        let mut tickets = apm_core::ticket::load_all_from_git(&root, &config.tickets.dir)?;
        let resolved = apm_core::ticket::resolve_id_in_slice(&tickets, &id)?;
        let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == resolved) else {
            anyhow::bail!("not found");
        };
        let now = chrono::Utc::now();
        apm_core::ticket::handoff(t, &agent_name, now)?;
        let branch = t.frontmatter.branch.clone()
            .or_else(|| apm_core::git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{resolved}"));
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );
        let content = t.serialize()?;
        apm_core::git::commit_to_branch(&root, &branch, &rel_path, &content,
            &format!("ticket({resolved}): reassign agent to {agent_name}"))?;
        // Re-load to return fresh state
        let tickets = apm_core::ticket::load_all_from_git(&root, &config.tickets.dir)?;
        Ok::<_, anyhow::Error>(tickets.into_iter().find(|t| t.frontmatter.id == resolved))
    }).await.unwrap();

    match result {
        Ok(Some(t)) => build_ticket_response(&t, &state.config).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) if e.to_string().contains("not found") =>
            (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
        Err(e) =>
            (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
```

Register in `main.rs`:
```rust
.route("/api/tickets/:id/take", post(take_ticket))
```

Note: `build_ticket_response` is the existing helper that serialises a ticket to `TicketResponse` (including `valid_transitions`). Extract it as a function if it isn't already.

---

**4. WorkerActivityPanel — Stop button**

In `apm-ui/src/components/WorkerActivityPanel.tsx`:

- Add `branch: string` to the `WorkerInfo` TypeScript interface
- Add a "Stop" column to the table header
- For each row where `status === 'running'`, render:

```tsx
<Button size="sm" variant="destructive" disabled={stopping === worker.pid}
  onClick={() => handleStop(worker.pid)}>
  Stop
</Button>
```

- `stopping` is a `useState<number | null>(null)` tracking which PID has an in-flight DELETE
- `handleStop(pid)`:
  1. Set `stopping = pid`
  2. `await fetch('/api/workers/' + pid, { method: 'DELETE' })`
  3. On success (204): call `queryClient.invalidateQueries({ queryKey: ['workers'] })`
  4. On error: set an inline `stopError` state and display it
  5. Always: clear `stopping`

- Crashed workers show no Stop button (their process is already gone)
- Add a `onKeyDown` handler on each row (`tabIndex={0}` to make rows focusable): when `event.shiftKey && event.key === 'K'` and the row's worker is running, call `handleStop(worker.pid)`

---

**5. TicketDetail — Reassign button**

In `apm-ui/src/components/TicketDetail.tsx`:

Add a "Reassign to me" button alongside the existing transition buttons (or in the same footer bar):

```tsx
<Button size="sm" variant="outline" disabled={reassigning}
  onClick={handleReassign}>
  Reassign to me
</Button>
```

- `handleReassign`:
  1. `setReassigning(true)`
  2. `POST /api/tickets/:id/take` (no body needed)
  3. On success: `queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })` and `queryClient.invalidateQueries({ queryKey: ['tickets'] })`
  4. On error: show inline error string
  5. Always: `setReassigning(false)`

---

**6. File changes summary**

Backend:
- `apm-server/src/routes/workers.rs` — add `branch` to WorkerInfo, add `delete_worker` handler + `stop_worker_by_pid` helper
- `apm-server/src/routes/tickets.rs` — add `take_ticket` handler, extract `build_ticket_response` helper if not already a function
- `apm-server/src/main.rs` — register `DELETE /api/workers/:pid` and `POST /api/tickets/:id/take`

Frontend:
- `apm-ui/src/components/WorkerActivityPanel.tsx` — add `branch` to type, add Stop button column
- `apm-ui/src/components/TicketDetail.tsx` — add Reassign to me button

---

**apm-core name verification (do before writing any handler):** Before implementing steps 2 and 3, verify that the following identifiers exist in `apm-core` under exactly these names:
- `apm_core::start::resolve_agent_name()` — resolves APM_AGENT_NAME or USER
- `apm_core::git::list_ticket_worktrees(root)` — returns worktree paths and branch names
- `apm_core::ticket::handoff(ticket, agent, now)` — updates the agent field on a ticket

If any of these are missing or named differently, add or rename them in `apm-core` first (as a separate commit) before wiring them into the server routes.

---

**Ordering note:** The `stop_worker_by_pid` function does filesystem I/O and process signals — always call it inside `tokio::task::spawn_blocking`. Same pattern as the existing workers handler. `ticket::handoff` also does git I/O — same treatment.

### Open questions



### Amendment requests

- [x] Add Acceptance Criterion: pressing `Shift+K` while a worker row is focused triggers the same Stop action as clicking the Stop button
- [x] Add note to Approach: the implementing agent must verify that `apm_core::start::resolve_agent_name()`, `apm_core::git::list_ticket_worktrees()`, and `ticket::handoff()` exist under exactly these names before writing the handler — they may need to be added to or renamed in apm-core first

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:29Z | new | in_design | philippepascal |
| 2026-03-31T07:34Z | in_design | specd | claude-0331-0730-b7f2 |
| 2026-03-31T18:15Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:12Z | ammend | in_design | philippepascal |
| 2026-03-31T19:16Z | in_design | specd | claude-0331-1430-c9d2 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T06:39Z | ready | in_progress | philippepascal |
| 2026-04-01T06:47Z | in_progress | implemented | claude-0401-0639-6510 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |