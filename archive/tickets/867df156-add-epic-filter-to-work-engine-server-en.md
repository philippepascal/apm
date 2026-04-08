+++
id = "867df156"
title = "Add epic filter to work engine server endpoint"
state = "closed"
priority = 4
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "73276"
branch = "ticket/867df156-add-epic-filter-to-work-engine-server-en"
created_at = "2026-04-01T21:56:02.797958Z"
updated_at = "2026-04-02T19:07:18.153896Z"
+++

## Spec

### Problem

The work engine server (POST /api/work/start) accepts no request body and always starts the engine in open mode -- dispatching any actionable ticket regardless of epic membership. There is no way for the UI (or any API caller) to start the engine in epic-exclusive mode, where only tickets belonging to a specific epic are dispatched.

Correspondingly, GET /api/work/status returns only a status field and has no way to communicate whether an active engine is running in epic-exclusive mode, which makes the UI unable to reflect that constraint to the user.

The design for epic-scoped scheduling is specified in docs/epics.md (section: Work engine -- epic filter). This ticket implements that slice: the server-side plumbing connecting an optional "epic" field in the start request through to run_engine_loop and spawn_next_worker, and the corresponding status reporting.

### Acceptance criteria

- [x] POST /api/work/start with an empty body starts the engine and returns a status response (no regression)
- [x] POST /api/work/start with body {"epic": "ab12cd34"} starts the engine without error
- [x] When the engine is started with {"epic": "ab12cd34"}, spawn_next_worker only considers tickets whose frontmatter.epic == "ab12cd34"
- [x] When the engine is started without an epic field, spawn_next_worker considers all actionable tickets (open mode, no regression)
- [x] GET /api/work/status returns {"status": "idle", "epic": "ab12cd34"} when the engine was started with that epic filter
- [x] GET /api/work/status returns {"status": "idle", "epic": null} when the engine was started without an epic filter
- [x] GET /api/work/status returns {"status": "stopped"} (no epic key) when no engine is running

### Out of scope

- apm epic CLI commands (apm epic new, list, show, close)
- Epic CRUD API routes (GET/POST /api/epics, GET /api/epics/:id)
- depends_on scheduling
- target_branch / PR targeting for epic tickets
- CreateTicketRequest epic or depends_on fields
- apm new --epic flag
- Balanced / multi-epic concurrent scheduling
- Frontend / UI changes
- apm work --epic CLI flag (separate from the server endpoint)

### Approach

Four files change. Order of changes matters for compilation.

1. apm-core/src/ticket.rs -- add epic field to Frontmatter
   Add after the focus_section field:
     #[serde(skip_serializing_if = "Option::is_none")]
     pub epic: Option<String>,
   This is the only change needed here; serde(flatten) in TicketResponse
   propagates it to all existing API responses automatically.

2. apm-core/src/start.rs -- add epic filter to spawn_next_worker
   Change signature:
     pub fn spawn_next_worker(root, no_aggressive, skip_permissions, epic: Option<&str>)
   After loading tickets and before calling pick_next, filter candidates:
     let tickets: Vec<Ticket> = match epic {
         Some(eid) => tickets.into_iter()
             .filter(|t| t.frontmatter.epic.as_deref() == Some(eid))
             .collect(),
         None => tickets,
     };
   Pass the (now owned) filtered vec to pick_next.
   No other callers of spawn_next_worker exist outside apm-core/src/work.rs,
   so no other call sites need updating.

3. apm-core/src/work.rs -- thread epic through run_engine_loop
   Change signature:
     pub fn run_engine_loop(root, cancel, interval_secs, max_concurrent, skip_permissions, epic: Option<String>)
   Store epic locally; pass epic.as_deref() to spawn_next_worker on each call.

4. apm-server/src/work.rs -- request parsing, engine state, and status response
   a. Add StartWorkRequest struct:
        #[derive(serde::Deserialize, Default)]
        pub struct StartWorkRequest { pub epic: Option<String> }
   b. Add epic: Option<String> field to WorkEngine struct.
   c. Change post_work_start signature to accept Option<Json<StartWorkRequest>>.
      Extract epic = body.and_then(|b| b.0.epic).
      Pass epic.clone() to run_engine_loop (last arg).
      Store epic in the WorkEngine struct.
      Note: Option<Json<T>> in Axum returns None when the body is absent or
      the Content-Type is not application/json, which preserves backward
      compat with the existing test that sends an empty body.
   d. Change get_work_status to read epic from the engine guard in the same
      lock used to check engine_is_alive:
        let (alive, epic) = {
            let guard = state.work_engine.lock().await;
            match guard.as_ref() {
                Some(e) => (engine_is_alive(e), e.epic.clone()),
                None => (false, None),
            }
        };
      Return {"status": "stopped"} (no epic key) when not alive.
      Include "epic": epic in running/idle responses.

5. Tests -- update existing tests and add new ones (inline in apm-server/src/work.rs)
   - Existing work_start_without_git_root_returns_stopped: no change needed
     (sends empty body -> Option<Json> -> None -> no epic filter -> same path).
   - Add: work_start_with_epic_field_accepted -- POST with JSON body {"epic":"abc123"}, expect 200.
   - Add: work_status_includes_epic_null_when_stopped -- confirm no "epic" key when stopped.
   - The epic-filter integration (spawn_next_worker filtering) is best tested
     in apm-core/src/start.rs unit tests: add a test that constructs tickets
     with and without epic frontmatter and verifies pick_next only sees the
     right ones when the filter is active.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:52Z | groomed | in_design | philippepascal |
| 2026-04-02T00:57Z | in_design | specd | claude-0402-0100-b7e2 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T06:27Z | ready | in_progress | philippepascal |
| 2026-04-02T06:30Z | in_progress | implemented | claude-0401-2300-c9d2 |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |