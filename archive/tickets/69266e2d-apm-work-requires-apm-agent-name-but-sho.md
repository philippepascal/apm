+++
id = "69266e2d"
title = "apm work requires APM_AGENT_NAME but should not"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/69266e2d-apm-work-requires-apm-agent-name-but-sho"
created_at = "2026-03-30T06:11:19.569472Z"
updated_at = "2026-03-30T18:08:36.098666Z"
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
agent), but until then `apm work` should auto-set a fallback name rather than
failing.

The same problem applies to `apm start <id>` and `apm start --next` for a
supervisor who hasn't exported `APM_AGENT_NAME`. These commands have no
inherent reason to require it: a sensible fallback (e.g. `$USER`) is better
than a hard failure. The env var remains the preferred way to set a name, but
it should not be mandatory.

### Acceptance criteria

- [x] `apm work` with `APM_AGENT_NAME` unset does not print `warning: dispatch failed: APM_AGENT_NAME is not set`
- [x] `apm work` with `APM_AGENT_NAME` unset dispatches workers for actionable tickets without error
- [x] Tickets started via `apm work` without `APM_AGENT_NAME` have a non-empty `agent` field in their frontmatter
- [x] `apm start <id>` with `APM_AGENT_NAME` unset succeeds and sets a non-empty `agent` field in the ticket
- [x] `apm start --next` with `APM_AGENT_NAME` unset succeeds and sets a non-empty `agent` field in the ticket
- [x] When `APM_AGENT_NAME` is set, all three commands (`apm work`, `apm start <id>`, `apm start --next`) use it as the agent name

### Out of scope

- Using PID as agent name for workers (covered by ticket `9baf1ac2`)
- Any UI for surfacing the fallback name beyond the existing `Agent name: <name>` print
- Making `APM_AGENT_NAME` truly optional as a long-term design choice — the env var remains the expected way to set identity; this ticket only adds a fallback to remove hard failures

### Approach

**Root cause:** `start::run()` reads `APM_AGENT_NAME` from the environment unconditionally (lines 7–8 in `start.rs`), bailing if unset. It is called from three paths: CLI (`main.rs`), `run_next()`, and `spawn_next_worker()` in `work.rs`. All three paths fail hard today if the env var is absent.

**Fix in two parts:**

**Part 1 — shared fallback helper** (`apm/src/cmd/start.rs`)

Add a small free function that centralises agent-name resolution:

```rust
fn resolve_agent_name() -> String {
    std::env::var("APM_AGENT_NAME")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))   // Windows compat
        .unwrap_or_else(|_| "apm".to_string())
}
```

Priority: `APM_AGENT_NAME` → `$USER` → `$USERNAME` → literal `"apm"`.

**Part 2 — thread `agent_name` as an explicit parameter through `run()`**

1. **`start::run()` signature change** (`apm/src/cmd/start.rs`)
   - Remove `let agent_name = std::env::var("APM_AGENT_NAME").map_err(…)?;` from line 7–8
   - Add `agent_name: &str` as the last parameter

2. **CLI dispatch** (`apm/src/main.rs`)
   - For `Command::Start { id: Some(id), … }`: call `resolve_agent_name()` and pass the result to `cmd::start::run()`

3. **`run_next()`** (`apm/src/cmd/start.rs`)
   - Replace the hard `APM_AGENT_NAME` read at line 180 with a call to `resolve_agent_name()`
   - Pass the result to `run()`

4. **`spawn_next_worker()`** (inside `run()` when `spawn = true`)
   - The spawn path already generates its own `worker_name` at line 133 (`format!("claude-{}-{:04x}", …)`), which is set on `cmd.env("APM_AGENT_NAME", &worker_name)` before launching the subprocess — this is correct and unchanged
   - The `agent_name` passed in from the call site is used for the ticket frontmatter (the *orchestrator's* identity); the worker gets its own generated name

No changes to `work.rs`. No changes to the spawn-path worker naming.

### Open questions



### Amendment requests

- [x] Why would apm start without APM_AGENT_NAME fail? what is the alternative for a supervisor that wants to start a specific ticket or just the next?

  **Addressed:** There is no good reason for the hard failure. The spec now extends the fallback to all `apm start` paths via a shared `resolve_agent_name()` helper (`APM_AGENT_NAME` → `$USER` → `$USERNAME` → `"apm"`). A supervisor who hasn't exported the env var gets a sensible agent name automatically. `APM_AGENT_NAME` remains the preferred mechanism but is no longer required.


### Code review
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:11Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:20Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T06:23Z | in_design | specd | claude-0329-spec-writer |
| 2026-03-30T06:27Z | specd | ammend | apm |
| 2026-03-30T06:30Z | ammend | in_design | claude-0330-0245-main |
| 2026-03-30T06:33Z | in_design | specd | claude-0329-1200-spec1 |
| 2026-03-30T06:35Z | specd | ready | apm |
| 2026-03-30T06:35Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T06:44Z | in_progress | implemented | claude-0329-1200-spec1 |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |