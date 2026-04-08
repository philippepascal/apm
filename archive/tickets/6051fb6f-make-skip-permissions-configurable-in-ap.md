+++
id = "6051fb6f"
title = "make skip_permissions configurable in apm.toml for worker spawning"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "49447"
branch = "ticket/6051fb6f-make-skip-permissions-configurable-in-ap"
created_at = "2026-03-30T21:17:37.548290Z"
updated_at = "2026-03-31T05:05:10.511028Z"
+++

## Spec

### Problem

When spawning worker subprocesses, `--dangerously-skip-permissions` is only applied when the user explicitly passes `-P` on the CLI (`apm start --spawn -P` or `apm work -P`). There is no way to set this persistently in `apm.toml`.

For unattended operation — cron jobs, `apm work --daemon`, automated pipelines — the user always wants workers to run without permission prompts. Having to remember to pass `-P` every time is error-prone: forgetting it causes workers to stall silently waiting for a prompt that never comes.

`[agents]` in `apm.toml` should support a `skip_permissions = true` flag that makes all worker spawns default to `--dangerously-skip-permissions`, with the CLI `-P` flag remaining as an override for one-off invocations.

### Acceptance criteria

- [x] `[agents]` in `.apm/config.toml` accepts a `skip_permissions = true` field without parse errors
- [x] When `skip_permissions = true` is set, `apm start --spawn <id>` passes `--dangerously-skip-permissions` to the worker without requiring `-P` on the CLI
- [x] When `skip_permissions = true` is set, `apm start --next --spawn` passes `--dangerously-skip-permissions` to spawned workers without requiring `-P`
- [x] When `skip_permissions = true` is set, `apm work` daemon mode passes `--dangerously-skip-permissions` to all spawned workers without requiring `-P`
- [x] Passing `-P` on the CLI continues to work regardless of the config value (logical OR: either source enables the flag)
- [x] When the field is absent from config, default is `false` and behaviour is unchanged
- [x] Unit test: `skip_permissions` parses correctly in `AgentsConfig` and defaults to `false`

### Out of scope

- Per-ticket `skip_permissions` overrides in ticket frontmatter
- Any change to the semantics of `--dangerously-skip-permissions` itself
- Interaction with the container/Docker worker path (container workers already bypass the Claude permission model)

### Approach

1. Add `skip_permissions: bool` field with `#[serde(default)]` to `AgentsConfig` in `apm-core/src/config.rs`
2. Update `AgentsConfig::default()` to set `skip_permissions: false`
3. In `apm-core/src/start.rs`, load `config.agents.skip_permissions` and OR it with the `skip_permissions` parameter in `run()`, `run_next()`, and `spawn_next_worker()` — the effective value is `cli_flag || config.agents.skip_permissions`
4. Add a unit test in `config.rs` that verifies the field parses to `true` when set and defaults to `false` when absent

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T21:17Z | — | new | philippepascal |
| 2026-03-30T21:17Z | new | in_design | philippepascal |
| 2026-03-30T21:31Z | in_design | specd | claude-0330-2120-b7f2 |
| 2026-03-30T22:49Z | specd | ready | apm |
| 2026-03-30T22:49Z | ready | in_progress | philippepascal |
| 2026-03-30T22:52Z | in_progress | implemented | claude-0330-2310-f4a2 |
| 2026-03-30T23:54Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |