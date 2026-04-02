+++
id = "9b4d2c9c"
title = "UI shows configured max workers and allow override"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
agent = "48994"
branch = "ticket/9b4d2c9c-ui-shows-configured-max-workers-and-allo"
created_at = "2026-04-02T19:20:21.647921Z"
updated_at = "2026-04-02T19:22:24.726022Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T19:20Z | — | new | apm-ui |
| 2026-04-02T19:20Z | new | groomed | apm |
| 2026-04-02T19:22Z | groomed | in_design | philippepascal |