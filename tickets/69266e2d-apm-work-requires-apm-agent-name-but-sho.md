+++
id = "69266e2d"
title = "apm work requires APM_AGENT_NAME but should not"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/69266e2d-apm-work-requires-apm-agent-name-but-sho"
created_at = "2026-03-30T06:11:19.569472Z"
updated_at = "2026-03-30T06:20:24.465866Z"
+++

## Spec

### Problem

`apm work` fails with `warning: dispatch failed: APM_AGENT_NAME is not set`
when the env var is not exported. `apm work` itself never uses `APM_AGENT_NAME`
— the error originates in `spawn_next_worker` → `apm start`, which reads it to
set the ticket's `agent` field.

`apm work` is designed to run headlessly (e.g. in CI or a background session)
where there is no human agent name. Requiring `APM_AGENT_NAME` breaks that use
case. This will be fully resolved by ticket `9baf1ac2` (workers use PID as
agent), but until then `apm work` should auto-set a fallback name (e.g.
`apm-work`) rather than failing silently with a warning.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:11Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:20Z | new | in_design | claude-0330-0245-main |
