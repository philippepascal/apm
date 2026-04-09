+++
id = "9b4d2c9c"
title = "UI shows configured max workers and allow override"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "apm-ui"
agent = "10297"
branch = "ticket/9b4d2c9c-ui-shows-configured-max-workers-and-allo"
created_at = "2026-04-02T19:20:21.647921Z"
updated_at = "2026-04-02T20:43:14.531431Z"
+++

## Spec

### Problem

The Work Engine Controls UI does not display the currently configured `agents.max_concurrent` value from `.apm/config.toml`. Users have no way to see how many workers the engine will spawn, and no way to change that number without manually editing the config file.

The problem has two parts: (1) the UI omits the value entirely, and (2) even if a UI control existed, there is no API endpoint to persist a change back to the config file. `post_work_start` reads `config.agents.max_concurrent` fresh on each start — so a runtime override that does not write to the file has no effect on the next start.

### Acceptance criteria

- [x] `WorkEngineControls` fetches `GET /api/agents/config` on mount and displays the `max_concurrent` value as read-only text (always, regardless of engine state)
- [x] `GET /api/agents/config` returns `{"max_concurrent": N, "override": N | null}` where `max_concurrent` is the value from `.apm/config.toml` (defaulting to 3 when absent) and `override` is the current in-memory override (null if none set)
- [x] The UI displays the effective worker count (override if set, otherwise configured value) in an editable field when the engine is stopped; the configured value is always shown as read-only alongside it
- [x] When the engine is stopped, clicking the effective worker count opens an inline number field (min 1, max 99) pre-filled with the current effective value
- [x] Pressing Enter or blurring the field with a valid value calls `PATCH /api/agents/config` with `{"override": N}`
- [x] `PATCH /api/agents/config` stores the override in `AppState` memory only — no file is written — and the override is lost when the apm-server restarts
- [x] After a successful PATCH, the displayed effective value updates to the new number
- [x] `post_work_start` uses the in-memory override value when present, falling back to `config.agents.max_concurrent` when no override is set
- [x] `PATCH /api/agents/config` with a value less than 1 or a non-integer returns HTTP 422
- [x] When the engine is running or idle, the effective worker count field is read-only (no click-to-edit)

### Out of scope

- Writing any config change to disk (all overrides are in-memory only)
- Editing any other `[agents]` config fields (instructions, skip_permissions, side_tickets)
- Editing any config section other than `[agents]`
- Showing or editing max_concurrent when the engine is started via the CLI (only the UI is covered)
- Validation that max_concurrent does not exceed available system resources
- Undo / history of override changes
- Persisting the override across server restarts

### Approach

**Backend — apm-server/src/agents.rs (new file)**

1. Add `max_concurrent_override: Arc<Mutex<Option<usize>>>` field to `AppState` in main.rs, initialized to `None`.

2. Add `GET /api/agents/config` handler:
   - Load `Config` from `state.git_root()` (same pattern as `post_work_start`)
   - Read `state.max_concurrent_override` from the mutex
   - Return `{"max_concurrent": N, "override": override_or_null}`
   - When `git_root` is `None` (in-memory mode), return the compiled-in default (3)

3. Add `PATCH /api/agents/config` handler:
   - Accept `{"override": usize}`; return 422 if value < 1
   - Write the value into `state.max_concurrent_override` (no disk write)
   - Return `{"max_concurrent": N, "override": N}` on success

4. Update `post_work_start` in work.rs:
   - After loading config, read `state.max_concurrent_override`
   - Use override if `Some(n)`, otherwise use `config.agents.max_concurrent.max(1)`
   - Same change needed in `get_work_dry_run` which also reads `max_concurrent`

5. Register routes in `build_app()` in main.rs:
   `.route("/api/agents/config", get(agents::get_agents_config).patch(agents::patch_agents_config))`

6. Add unit tests in agents.rs:
   - GET with no override returns configured default (3)
   - PATCH stores override; subsequent GET returns updated override
   - PATCH with 0 returns 422
   - POST /api/work/start uses override when set

**Frontend — apm-ui/src/components/WorkEngineControls.tsx**

1. Add `fetchAgentsConfig`: `GET /api/agents/config -> {max_concurrent: number, override: number | null}`
2. Add `patchAgentsConfig(n)`: `PATCH /api/agents/config` with body `{override: n}`
3. Add `useQuery(['agents-config'], fetchAgentsConfig)`
4. Add `useMutation` that calls `patchAgentsConfig` and invalidates `['agents-config']` on success
5. Render in the existing flex row:
   - Always: read-only label showing configured `max_concurrent` (e.g. "config: 3")
   - Effective value using `InlineNumberField` (label="workers", min=1, max=99):
     - When engine is stopped: editable; `onCommit` fires the mutation
     - When engine is running or idle: rendered as read-only plain text (no click target)
   - Effective value = `override ?? max_concurrent`

**Key constraints:**
- No disk writes — the override is purely in-memory `AppState`
- `InlineNumberField` already exists in `apm-ui/src/components/InlineNumberField.tsx` — do not duplicate it
- No new crate dependencies needed

### Open questions


### Amendment requests

- [x] Incorrect feature: we don't want the UI to be able to change the configured max workers. The configured max worker is always visible as readonly. 
  We want the UI to be able to locally (in the apm-server) use a override value instead of the configured value. That override is only for the apm-server,
  lives in memory and is lost when the apm-server is closed.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T19:20Z | — | new | apm-ui |
| 2026-04-02T19:20Z | new | groomed | apm |
| 2026-04-02T19:22Z | groomed | in_design | philippepascal |
| 2026-04-02T19:26Z | in_design | specd | claude-0402-1930-sp9x |
| 2026-04-02T20:05Z | specd | ammend | apm |
| 2026-04-02T20:05Z | ammend | in_design | philippepascal |
| 2026-04-02T20:08Z | in_design | specd | claude-0402-2010-sp9x |
| 2026-04-02T20:11Z | specd | ready | apm |
| 2026-04-02T20:11Z | ready | in_progress | philippepascal |
| 2026-04-02T20:17Z | in_progress | implemented | claude-0402-2015-wk7z |
| 2026-04-02T20:43Z | implemented | closed | apm-sync |