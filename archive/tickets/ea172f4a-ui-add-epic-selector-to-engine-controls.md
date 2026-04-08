+++
id = "ea172f4a"
title = "UI: add epic selector to engine controls"
state = "closed"
priority = 2
effort = 4
risk = 2
author = "claude-0401-2145-a8f3"
agent = "17152"
branch = "ticket/ea172f4a-ui-add-epic-selector-to-engine-controls"
created_at = "2026-04-01T21:56:28.916880Z"
updated_at = "2026-04-02T19:08:10.370559Z"
+++

## Spec

### Problem

The engine controls panel in the UI has no way to start the engine in epic-exclusive mode, and when exclusive mode is active there is no visual indicator of which epic is running. Without this, the UI cannot drive focused epic sprints.

Currently `WorkEngineControls.tsx` exposes a plain Start/Stop toggle with no parameters. The desired behaviour is:

1. Before starting: show an optional **Epic** selector dropdown (populated from `GET /api/epics`) so the user can choose to restrict the engine to one epic.
2. While running in exclusive mode: display a small `epic: <slug>` label that links to the epic filter on the supervisor board.

This requires extending the server's work engine API to accept and remember an optional epic filter, implementing a minimal `GET /api/epics` route, and adding the `epic` optional field to `Frontmatter` so the engine loop can filter on it.

### Acceptance criteria

- [x] `POST /api/work/start` with body `{"epic": "ab12cd34"}` starts the engine in exclusive mode
- [x] `POST /api/work/start` with no body (or body without `epic`) starts the engine in open mode, identical to current behaviour
- [x] `GET /api/work/status` includes `"epic": "ab12cd34"` when the engine is running in exclusive mode
- [x] `GET /api/work/status` includes `"epic": null` when the engine is running in open mode
- [x] The engine controls panel shows an Epic dropdown when the engine is stopped
- [x] The Epic dropdown is populated with epics from `GET /api/epics`; a blank/"All" option is present as the default (open mode)
- [x] Clicking Start with an epic selected sends `{"epic": "<id>"}` in the start request body
- [x] Clicking Start with no epic selected sends no epic in the start request body
- [x] When the engine is running in exclusive mode, a label `epic: <slug>` is shown next to the status badge
- [x] The `epic: <slug>` label is not shown when the engine is running in open mode
- [x] When the engine is stopped or idle the Epic dropdown is visible; when running it is hidden (replaced by the label if applicable)

### Out of scope

- Epic commands (`apm epic new`, `apm epic list`, `apm epic show`, `apm epic close`)
- `POST /api/epics` (create) and `GET /api/epics/:id` (detail) server routes
- `depends_on` scheduling and lock-icon UI on ticket cards
- Epic column and filter dropdown on the queue panel
- Epic filter on the supervisor board filter bar
- Epic and depends-on fields in the new-ticket modal
- Epic and depends-on display in the ticket detail panel
- `target_branch` frontmatter field and worktree provisioning from epic branch
- `derived state` computation (in_progress / done / etc.) on epics — the list endpoint returns title, branch, and ID only

### Approach

Prerequisite: ticket 54b043f7 must be merged before this ticket — it implements the `GET /api/epics` route that this ticket's UI consumes.

1. **apm-core/src/ticket.rs** — add `epic` to `Frontmatter`

   Add optional field (serde default so existing tickets without the field deserialise cleanly):
   ```
   pub epic: Option<String>
   ```

2. **apm-core/src/work.rs** — thread epic filter through engine loop

   Change `run_engine_loop` signature to accept `epic_filter: Option<String>`.
   Add `epic_filter: Option<&str>` to `spawn_next_worker`. After loading tickets, when `epic_filter` is `Some(id)`, filter slice to keep only tickets where `frontmatter.epic.as_deref() == Some(id)`, then call `pick_next` on the filtered slice.

3. **apm-server/src/work.rs** — extend `WorkEngine`, status, and start

   - `WorkEngine` struct: add `epic: Option<String>` field to remember the filter.
   - `get_work_status`: read `engine.epic.clone()` and include it in the JSON response as `"epic"` key (`null` when open mode).
   - `post_work_start`: accept optional JSON body via axum `Option<Json<StartRequest>>` extractor where `StartRequest` has an optional `epic: Option<String>` field. Pass epic to `run_engine_loop` and store it on `WorkEngine`.

4. **apm-ui/src/components/WorkEngineControls.tsx**

   - Add `useQuery` for epics calling `GET /api/epics` (route provided by ticket 54b043f7)
   - Add `selectedEpic` local state string (empty = open mode)
   - Extend `fetchStatus` return type to include `epic: string | null`
   - When stopped: show Epic `<select>` before Start button (blank option + one per epic)
   - When running/idle: hide select; if `status.epic` non-null show `epic: <slug>` label linking to `/?epic=<id>`
   - `startEngine` accepts optional epic param; include in POST body when non-empty

Order of changes: `ticket.rs` → core `work.rs` → server `work.rs` → `WorkEngineControls.tsx`

Tests:
- Server: `POST /api/work/start` with epic param → `GET /api/work/status` returns that epic
- All `cargo test --workspace` tests pass

### Open questions


### Amendment requests

- [x] Remove step 4 from Approach (GET /api/epics route implementation) — that route is owned by ticket 54b043f7. This ticket should declare 54b043f7 as a prerequisite and consume the existing route, not re-implement it.
- [x] Remove the two AC items that test GET /api/epics behaviour — they belong to 54b043f7. The only AC items for this ticket should be about the UI epic selector and the engine start/status API fields.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:58Z | groomed | in_design | philippepascal |
| 2026-04-02T01:02Z | in_design | specd | claude-0401-2333-spec1 |
| 2026-04-02T01:37Z | specd | ammend | philippepascal |
| 2026-04-02T01:43Z | ammend | in_design | philippepascal |
| 2026-04-02T01:45Z | in_design | specd | claude-0401-2200-spec2 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T07:00Z | ready | in_progress | philippepascal |
| 2026-04-02T07:04Z | in_progress | implemented | claude-0402-0800-w1f2 |
| 2026-04-02T19:08Z | implemented | closed | apm-sync |