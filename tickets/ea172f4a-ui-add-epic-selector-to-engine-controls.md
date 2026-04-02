+++
id = "ea172f4a"
title = "UI: add epic selector to engine controls"
state = "in_design"
priority = 2
effort = 4
risk = 0
author = "claude-0401-2145-a8f3"
agent = "87245"
branch = "ticket/ea172f4a-ui-add-epic-selector-to-engine-controls"
created_at = "2026-04-01T21:56:28.916880Z"
updated_at = "2026-04-02T01:01:56.874685Z"
+++

## Spec

### Problem

The engine controls panel in the UI has no way to start the engine in epic-exclusive mode, and when exclusive mode is active there is no visual indicator of which epic is running. Without this, the UI cannot drive focused epic sprints.

Currently `WorkEngineControls.tsx` exposes a plain Start/Stop toggle with no parameters. The desired behaviour is:

1. Before starting: show an optional **Epic** selector dropdown (populated from `GET /api/epics`) so the user can choose to restrict the engine to one epic.
2. While running in exclusive mode: display a small `epic: <slug>` label that links to the epic filter on the supervisor board.

This requires extending the server's work engine API to accept and remember an optional epic filter, implementing a minimal `GET /api/epics` route, and adding the `epic` optional field to `Frontmatter` so the engine loop can filter on it.

### Acceptance criteria

- [ ] `GET /api/epics` returns a JSON array of epic objects with at least `id`, `title`, and `branch` fields, derived from `epic/*` remote git branches
- [ ] `GET /api/epics` returns an empty array when no `epic/*` branches exist
- [ ] `POST /api/work/start` with body `{"epic": "ab12cd34"}` starts the engine in exclusive mode
- [ ] `POST /api/work/start` with no body (or body without `epic`) starts the engine in open mode, identical to current behaviour
- [ ] `GET /api/work/status` includes `"epic": "ab12cd34"` when the engine is running in exclusive mode
- [ ] `GET /api/work/status` includes `"epic": null` when the engine is running in open mode
- [ ] The engine controls panel shows an Epic dropdown when the engine is stopped
- [ ] The Epic dropdown is populated with epics from `GET /api/epics`; a blank/"All" option is present as the default (open mode)
- [ ] Clicking Start with an epic selected sends `{"epic": "<id>"}` in the start request body
- [ ] Clicking Start with no epic selected sends no epic in the start request body
- [ ] When the engine is running in exclusive mode, a label `epic: <slug>` is shown next to the status badge
- [ ] The `epic: <slug>` label is not shown when the engine is running in open mode
- [ ] When the engine is stopped or idle the Epic dropdown is visible; when running it is hidden (replaced by the label if applicable)

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

1. apm-core/src/ticket.rs - add epic to Frontmatter

Add optional field to Frontmatter (serde default so existing tickets without field deserialise cleanly):
  pub epic: Option<String>

2. apm-core/src/work.rs - thread epic filter through engine loop

Change run_engine_loop signature to accept epic_filter: Option<String>.
Add epic_filter: Option<&str> to spawn_next_worker. After loading tickets, when epic_filter is Some(id),
filter slice to keep only tickets where frontmatter.epic.as_deref() == Some(id), then call pick_next on
the filtered slice.

3. apm-server/src/work.rs - extend WorkEngine, status, and start

WorkEngine struct: add epic: Option<String> field to remember the filter.

get_work_status: read engine.epic.clone() and include it in the JSON response as "epic" key (null when open mode).

post_work_start: accept optional JSON body via axum Option<Json<StartRequest>> extractor where
StartRequest has an optional epic: Option<String> field. Pass epic to run_engine_loop and store it on WorkEngine.

4. apm-server - GET /api/epics route (new handler, can live in main.rs or new epics.rs)

- Run git branch -r in git_root
- Filter lines matching origin/epic/ prefix
- Parse id (first 8 chars after prefix) and title (remainder with hyphens to spaces, title-cased)
- Return Vec<EpicSummary> with id, title, branch fields
- InMemory source returns empty array
- Register .route("/api/epics", get(get_epics))

5. apm-ui/src/components/WorkEngineControls.tsx

- Add useQuery for epics calling GET /api/epics
- Add selectedEpic local state string (empty = open mode)
- Extend fetchStatus return type to include epic: string | null
- When stopped: show Epic select before Start button (blank option + one per epic)
- When running/idle: hide select; if status.epic non-null show label with link to /?epic=<id>
- startEngine accepts optional epic param; include in POST body when non-empty

Order of changes: ticket.rs -> core work.rs -> server work.rs -> epics route -> WorkEngineControls.tsx

Tests:
- Unit: branch-name parser extracts id and title correctly
- Server: GET /api/epics on in-memory returns empty array
- Server: POST /api/work/start with epic param -> GET /api/work/status returns that epic
- All cargo test --workspace tests pass

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:58Z | groomed | in_design | philippepascal |