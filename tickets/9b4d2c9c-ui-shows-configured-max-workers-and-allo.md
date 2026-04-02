+++
id = "9b4d2c9c"
title = "UI shows configured max workers and allow override"
state = "ammend"
priority = 0
effort = 4
risk = 2
author = "apm-ui"
agent = "48994"
branch = "ticket/9b4d2c9c-ui-shows-configured-max-workers-and-allo"
created_at = "2026-04-02T19:20:21.647921Z"
updated_at = "2026-04-02T20:05:05.187118Z"
+++

## Spec

### Problem

The Work Engine Controls UI does not display the currently configured `agents.max_concurrent` value from `.apm/config.toml`. Users have no way to see how many workers the engine will spawn, and no way to change that number without manually editing the config file.

The problem has two parts: (1) the UI omits the value entirely, and (2) even if a UI control existed, there is no API endpoint to persist a change back to the config file. `post_work_start` reads `config.agents.max_concurrent` fresh on each start — so a runtime override that does not write to the file has no effect on the next start.

### Acceptance criteria

- [ ] `WorkEngineControls` fetches `GET /api/agents/config` on mount and displays the `max_concurrent` value
- [ ] `GET /api/agents/config` returns `{"max_concurrent": N}` where N matches what is in `.apm/config.toml` (defaulting to 3 when absent)
- [ ] When the engine is stopped, clicking the displayed value opens an inline number field (min 1, max 99) pre-filled with the current value
- [ ] Pressing Enter or blurring the field with a valid value calls `PATCH /api/agents/config` with `{"max_concurrent": N}`
- [ ] After a successful PATCH, the displayed value updates to the new number
- [ ] `PATCH /api/agents/config` writes `max_concurrent` to the `[agents]` section of `.apm/config.toml` and the change survives a server restart
- [ ] `PATCH /api/agents/config` with a value less than 1 or a non-integer returns HTTP 422
- [ ] When the engine is running or idle, the max-workers field is read-only (no click-to-edit)

### Out of scope

- Editing any other `[agents]` config fields (instructions, skip_permissions, side_tickets)
- Editing any config section other than `[agents]`
- Showing or editing max_concurrent when the engine is started via the CLI (only the UI is covered)
- Validation that max_concurrent does not exceed available system resources
- Undo / history of config changes

### Approach

Backend — new apm-server/src/agents.rs

1. Add GET /api/agents/config handler:
   - Load Config from state.git_root() (same pattern as post_work_start)
   - Return {"max_concurrent": config.agents.max_concurrent.max(1)}
   - When git_root is None (in-memory), return the compiled-in default (3)

2. Add PATCH /api/agents/config handler:
   - Accept {"max_concurrent": usize}; return 422 if value < 1
   - Resolve config file path: .apm/config.toml if it exists, else apm.toml
   - Read the file as a toml::Value, set value["agents"]["max_concurrent"] to the new integer,
     serialize with toml::to_string_pretty, overwrite the file
   - If the [agents] table does not exist yet, insert it
   - Return {"max_concurrent": N} on success

3. Register routes in build_app() in main.rs:
     .route("/api/agents/config", get(agents::get_agents_config).patch(agents::patch_agents_config))

4. Add unit tests in agents.rs:
   - GET with no git root returns default (3)
   - PATCH persists value and GET returns updated value afterward
   - PATCH with 0 returns 422

Frontend — apm-ui/src/components/WorkEngineControls.tsx

1. Add fetchAgentsConfig: GET /api/agents/config -> {max_concurrent: number}
2. Add patchAgentsConfig(n): PATCH /api/agents/config with body {max_concurrent: n}
3. Add useQuery(['agents-config'], fetchAgentsConfig)
4. Add useMutation that calls patchAgentsConfig and invalidates ['agents-config'] on success
5. Render an InlineNumberField (label="workers", min=1, max=99) in the existing flex row:
   - When isEngineActive, render as read-only plain text (label + value, no click target)
   - When engine is stopped, onCommit fires the mutation

Key constraints:
- toml::Value round-trip will reformat the config file (comments and key ordering lost).
  This is acceptable — the config is machine-managed.
- No new crate dependencies needed; toml is already in the workspace.
- InlineNumberField is already implemented — do not duplicate it.

### Open questions


### Amendment requests
Incorrect feature: we don't want the UI to be able to change the configured max workers. The configured max worker is always visible as readonly. 
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
