+++
id = "0fa737ae"
title = "UI: change display of max workers"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0fa737ae-ui-change-display-of-max-workers"
created_at = "2026-04-28T19:24:12.894681Z"
updated_at = "2026-04-28T20:40:23.348502Z"
+++

## Spec

### Problem

The work engine controls UI currently shows `config: <max_concurrent>` — a single number. But the config actually carries three distinct limits: total max (`max_concurrent`), default-branch max (`max_workers_on_default`), and epic max (`max_workers_per_epic`). The display hides the per-branch and per-epic ceilings, so there is no way to tell from the UI what those values are without reading the config file directly.

Additionally, the label "workers" used for both the active static display and the editable field when the engine is stopped is ambiguous. "Override max" is more precise: it names what the control actually sets.

The fix is purely presentational: extend the API response to carry all three config values, update the config badge to show all three, and rename the "workers" label to "override max". No scheduling logic changes.

### Acceptance criteria

- [x] The config badge reads `config: t <total> d <default> e <epic>` using the three values from the API (e.g. `config: t 3 d 1 e 1`)
- [x] When the engine is inactive, the `InlineNumberField` carries the label "override max" instead of "workers"
- [x] When the engine is active, the static text reads `override max: <value>` instead of `workers: <value>`
- [x] The `GET /api/agents/config` response includes `max_workers_on_default` and `max_workers_per_epic` fields alongside the existing `max_concurrent` and `override`
- [x] When no git root is present (in-memory default), `max_workers_on_default` and `max_workers_per_epic` both default to `1`
- [x] Existing backend tests pass (or are updated to assert the new response shape)

### Out of scope

- Changes to work engine scheduling logic (which limits are enforced and how)
- Changes to the PATCH endpoint's behavior (how the override value is stored and applied)
- UI controls to edit per-epic or per-default-branch limits (read-only display only)
- Any change to the `apm-core` config struct or its defaults

### Approach

**`apm-server/src/agents.rs`**

- Add `max_workers_per_epic: usize` and `max_workers_on_default: usize` to `AgentsConfigResponse`.
- In `get_agents_config`: after loading config, also read `config.agents.max_workers_per_epic` and `config.agents.max_workers_on_default`; when no git root, default both to `1`. Include the two new fields in the returned `AgentsConfigResponse`.
- In `patch_agents_config`: same — load and return all three config values in the response.
- Update the test `get_agents_config_returns_default_when_in_memory` to assert `json["max_workers_on_default"] == 1` and `json["max_workers_per_epic"] == 1`.
- Update `patch_agents_config_stores_override` similarly.

**`apm-ui/src/components/WorkEngineControls.tsx`**

- Extend the `AgentsConfig` type (lines 43–46) with `max_workers_on_default: number` and `max_workers_per_epic: number`.
- Line 136: change the config badge from `config: {agentsConfig.max_concurrent}` to `config: t {agentsConfig.max_concurrent} d {agentsConfig.max_workers_on_default} e {agentsConfig.max_workers_per_epic}`.
- Line 139: change `label="workers"` on `InlineNumberField` to `label="override max"`.
- Line 146: change `workers: {agentsConfig.override ?? agentsConfig.max_concurrent}` to `override max: {agentsConfig.override ?? agentsConfig.max_concurrent}`.

The API change is purely additive (new fields on the response), so the server and client sides can land in either order without breakage.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:24Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:38Z | groomed | in_design | philippepascal |
| 2026-04-28T19:41Z | in_design | specd | claude-0428-1938-f518 |
| 2026-04-28T20:34Z | specd | ready | philippepascal |
| 2026-04-28T20:40Z | ready | in_progress | philippepascal |
