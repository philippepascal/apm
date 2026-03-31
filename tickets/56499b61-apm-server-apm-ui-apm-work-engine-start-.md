+++
id = "56499b61"
title = "apm-server + apm-ui: apm work engine start/stop controls"
state = "in_design"
priority = 0
effort = 5
risk = 4
author = "apm"
agent = "12358"
branch = "ticket/56499b61-apm-server-apm-ui-apm-work-engine-start-"
created_at = "2026-03-31T06:13:12.529756Z"
updated_at = "2026-03-31T19:16:46.067797Z"
+++

## Spec

### Problem

The apm work engine — which dispatches Claude worker agents to actionable tickets — can only be started or stopped from the command line. There is no way to control it from the UI.

This ticket adds three server endpoints (`GET /api/work/status`, `POST /api/work/start`, `POST /api/work/stop`) and a control widget at the top of the workerview left column. The widget shows the current engine state (running / idle / stopped) and a button to toggle it, with a keyboard shortcut.

The work engine runs as a child process of the axum server, equivalent to `apm work --daemon`. The server tracks the child process handle in shared state. Start spawns it; stop sends SIGTERM. Status is derived by checking whether the child is alive and whether any worker PID files exist.

### Acceptance criteria

- [ ] `GET /api/work/status` returns `{"status":"stopped"}` when no engine task is running
- [ ] `GET /api/work/status` returns `{"status":"running"}` when the engine task is alive and at least one worker PID file exists and the process is alive
- [ ] `GET /api/work/status` returns `{"status":"idle"}` when the engine task is alive but no active worker PID files exist
- [ ] `POST /api/work/start` starts the tokio engine task and returns the new status; a second call while already running returns the current status without starting a second task
- [ ] `POST /api/work/stop` signals the engine task to stop, waits for it to exit, and returns `{"status":"stopped"}`; calling it when already stopped returns `{"status":"stopped"}` without an error
- [ ] The WorkerView panel header shows a Start/Stop toggle button and a status badge labelled "Running", "Idle", or "Stopped"
- [ ] Clicking Start calls `POST /api/work/start`; the button and badge update to the returned state without a full page reload
- [ ] Clicking Stop calls `POST /api/work/stop`; the button and badge update to the returned state without a full page reload
- [ ] The status badge auto-refreshes at a poll interval of 5 s or less while the WorkerView panel is mounted
- [ ] A keyboard shortcut (`Shift+W`) toggles the engine start/stop from anywhere in the workscreen

### Out of scope

- Dry-run preview before starting (covered by the follow-on ticket, Step 12b)
- Per-worker stop controls (covered by Step 15 — worker management)
- Configuring `max_concurrent` or `interval_secs` from the UI
- Log tail viewer (covered by Step 14b)
- Authentication or access control on the work endpoints

### Approach

**Prerequisites:** This ticket requires `apm-server` (Step 1) and the `GET /api/workers` endpoint plus WorkerView panel (Step 7a) to already exist. The axum `AppState` struct from Step 7a must be extended with the daemon handle.

**Server-side — `apm-server/src/routes/work.rs` (new file)**

1. Extend `AppState` in `apm-server/src/main.rs` with:
   ```rust
   work_daemon: Arc<Mutex<Option<std::process::Child>>>
   ```
2. Add three handlers wired to the router:
   - `GET /api/work/status` → `get_work_status`
   - `POST /api/work/start` → `post_work_start`
   - `POST /api/work/stop` → `post_work_stop`
3. **Status logic** (shared helper `fn engine_status(state) -> &str`):
   - If child handle is `None` or `try_wait()` shows it exited → `"stopped"`
   - Else: scan worktrees for `.apm-worker.pid` files (reuse `git::list_ticket_worktrees` + `worker::is_alive`); if any alive → `"running"`, else → `"idle"`
4. **Start handler:** If daemon is already running (child alive), return current status. Otherwise run `std::process::Command::new("apm").args(["work", "--daemon"])` with `current_dir` set to the repo root from config, spawn it, store in state, return status.
5. **Stop handler:** If no child, return `stopped`. Otherwise send SIGTERM via `kill -TERM <pid>`, call `child.wait()` (with a timeout of ~5 s, then SIGKILL if necessary), set state to `None`, return `stopped`.

All three handlers return `{"status": "<value>"}` as JSON with HTTP 200.

**UI-side — `apm-ui/src/components/WorkEngineControls.tsx` (new component)**

1. Use TanStack Query's `useQuery` to poll `GET /api/work/status` every 3 s.
2. Use TanStack Query's `useMutation` for start and stop, with `onSuccess` calling `queryClient.invalidateQueries(['work-status'])`.
3. Render a shadcn/ui `Badge` for the status and a `Button` labelled "Start" or "Stop" depending on status.
4. Mount the component inside the WorkerView panel header row (already established by Step 7a).
5. Register the keyboard shortcut in the Zustand store's global key handler: `Ctrl+Shift+W` calls the appropriate mutation based on current status.

### Open questions



### Amendment requests

- [ ] Change keyboard shortcut in AC from `Ctrl+Shift+W` to `Shift+W` (keyboard spec uses Shift+W)
- [ ] Rewrite the Approach entirely — apm-server must NOT spawn the `apm` CLI as a subprocess. The server must be self-contained: implement the work dispatch loop directly in Rust using apm-core functions (ticket loading, actionable filtering, score sorting, state transitions, worktree provisioning) running inside a tokio background task. For spawning Claude worker agents, use the same logic as `apm start --spawn` but called via apm-core directly. Remove all references to `std::process::Command::new("apm")`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:02Z | new | in_design | philippepascal |
| 2026-03-31T07:05Z | in_design | specd | claude-0331-0800-b7f2 |
| 2026-03-31T18:14Z | specd | ammend | claude-0331-1200-a7b9 |
| 2026-03-31T19:16Z | ammend | in_design | philippepascal |