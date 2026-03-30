+++
id = "69266e2d"
title = "apm work requires APM_AGENT_NAME but should not"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/69266e2d-apm-work-requires-apm-agent-name-but-sho"
created_at = "2026-03-30T06:11:19.569472Z"
updated_at = "2026-03-30T06:23:26.927266Z"
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

- [ ] `apm work` with `APM_AGENT_NAME` unset does not print `warning: dispatch failed: APM_AGENT_NAME is not set`
- [ ] `apm work` with `APM_AGENT_NAME` unset dispatches workers for actionable tickets without error
- [ ] Tickets started via `apm work` without `APM_AGENT_NAME` have a non-empty `agent` field in their frontmatter
- [ ] `apm start <id>` without `APM_AGENT_NAME` still fails with `APM_AGENT_NAME is not set`
- [ ] `apm start --next` without `APM_AGENT_NAME` still fails with `APM_AGENT_NAME is not set`

### Out of scope

- Using PID as agent name for workers (covered by ticket `9baf1ac2`)
- Changing `apm start` or `apm start --next` behaviour for interactive callers
- Any UI for surfacing the fallback name beyond the existing `Agent name: <name>` print

### Approach

**Root cause:** `start::run()` reads `APM_AGENT_NAME` from the environment unconditionally (lines 7–8 in `start.rs`). It is called from three paths: CLI (`main.rs`), `run_next()`, and `spawn_next_worker()`. Only the third path is headless — `spawn_next_worker` already generates a `worker_name` (line 376) but does so *after* calling `run()`, so `run()` fails before that point.

**Fix:** Remove the `APM_AGENT_NAME` read from inside `run()` and instead accept `agent_name: &str` as an explicit parameter. Each call site supplies it:

1. **`start::run()` signature change** (`apm/src/cmd/start.rs`)
   - Remove `let agent_name = std::env::var("APM_AGENT_NAME").map_err(…)?;` from the top of `run()`
   - Add `agent_name: &str` as the last parameter

2. **`spawn_next_worker()`** (`apm/src/cmd/start.rs`)
   - Move the `worker_name` generation (currently line 376, after the `run()` call) to *before* the `run()` call
   - Pass `&worker_name` to `run()`
   - Remove the now-duplicate generation further down; reuse the same binding

3. **`run_next()`** (`apm/src/cmd/start.rs`)
   - Already reads `APM_AGENT_NAME` at line 180 — keep that check, pass `&agent_name` explicitly to `run()`

4. **CLI dispatch** (`apm/src/main.rs`)
   - For `Command::Start { id: Some(id), … }`: read `APM_AGENT_NAME` from env (bail if missing) and pass to `cmd::start::run()`
   - The `--next` path delegates entirely to `run_next()`, which handles its own env read

No changes to `work.rs`.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:11Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:20Z | new | in_design | claude-0330-0245-main |