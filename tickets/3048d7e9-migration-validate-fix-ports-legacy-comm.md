+++
id = "3048d7e9"
title = "Migration: validate --fix ports legacy command/args/model to agent + options"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3048d7e9-migration-validate-fix-ports-legacy-comm"
created_at = "2026-04-30T20:03:17.277300Z"
updated_at = "2026-04-30T21:36:57.594854Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["6cac8518"]
+++

## Spec

### Problem

Existing APM projects have a `.apm/config.toml` using the legacy `[workers]` shape: `command = "claude"`, `args = ["--print", ...]`, and `model = "sonnet"`. After upgrading to the agent-wrapper architecture (ticket 6cac8518), those projects receive a deprecation warning on every `apm start` invocation but have no automated way to migrate.

The desired state is `agent = "claude"` in `[workers]` with model moved to `[workers.options]` and `args` dropped entirely (the wrapper now owns CLI flag construction). A matching migration must apply to every `[worker_profiles.<X>]` section as well.

This ticket adds that migration to `apm validate --fix`. A developer who upgrades APM runs `apm validate --fix`, sees a one-line confirmation message, and their config is correct without any manual editing. If the project was using a non-Claude command, automated migration is not safe — the tool warns and stops so the user can hand-pick a wrapper.

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
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:36Z | groomed | in_design | philippepascal |