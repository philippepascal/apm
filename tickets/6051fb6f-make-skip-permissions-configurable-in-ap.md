+++
id = "6051fb6f"
title = "make skip_permissions configurable in apm.toml for worker spawning"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "20197"
branch = "ticket/6051fb6f-make-skip-permissions-configurable-in-ap"
created_at = "2026-03-30T21:17:37.548290Z"
updated_at = "2026-03-30T21:17:43.145746Z"
+++

## Spec

### Problem

When spawning worker subprocesses, `--dangerously-skip-permissions` is only applied when the user explicitly passes `-P` on the CLI (`apm start --spawn -P` or `apm work -P`). There is no way to set this persistently in `apm.toml`.

For unattended operation — cron jobs, `apm work --daemon`, automated pipelines — the user always wants workers to run without permission prompts. Having to remember to pass `-P` every time is error-prone: forgetting it causes workers to stall silently waiting for a prompt that never comes.

`[agents]` in `apm.toml` should support a `skip_permissions = true` flag that makes all worker spawns default to `--dangerously-skip-permissions`, with the CLI `-P` flag remaining as an override for one-off invocations.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T21:17Z | — | new | philippepascal |
| 2026-03-30T21:17Z | new | in_design | philippepascal |