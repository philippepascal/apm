+++
id = "37c280f2"
title = "apm take should work without APM_AGENT_NAME set"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "50164"
branch = "ticket/37c280f2-apm-take-should-work-without-apm-agent-n"
created_at = "2026-03-30T14:41:58.874046Z"
updated_at = "2026-03-30T18:08:21.719693Z"
+++

## Spec

### Problem

Currently, `apm take` hard-fails with `APM_AGENT_NAME is not set` if the environment variable is absent (take.rs lines 7–8). This is inconsistent with other commands: `apm state`, `apm close`, and `apm start` all fall back gracefully via `unwrap_or_else` or the `resolve_agent_name()` helper (APM_AGENT_NAME → $USER → $USERNAME → literal "apm").

`apm take` is typically used by supervisors or engineers reclaiming a stalled ticket. These callers may not have exported `APM_AGENT_NAME`. The hard failure forces an extra export step that has no real safety benefit, since the same agent-name resolution logic already exists in the codebase.

The desired behaviour is that `apm take` uses the same `resolve_agent_name()` helper from `start.rs` so it succeeds whenever any reasonable identity is available, and only produces a generic fallback when no env vars are set at all.

### Acceptance criteria

- [x] `apm take <id>` with `APM_AGENT_NAME` unset succeeds (exit 0) and completes the handoff
- [x] `apm take <id>` with `APM_AGENT_NAME` unset and `USER=alice` sets the ticket's `agent` field to `alice`
- [x] `apm take <id>` with `APM_AGENT_NAME` unset and neither `USER` nor `USERNAME` set falls back to agent name `apm`
- [x] `apm take <id>` with `APM_AGENT_NAME=my-agent` still uses `my-agent` (existing behaviour preserved)
- [x] `apm take <id>` when already assigned to the resolved agent name prints the no-op message without error

### Out of scope

- Making `APM_AGENT_NAME` truly optional across all commands — only `apm take` is addressed here
- Changing the fallback resolution order (that was established by ticket 69266e2d)
- Any change to `apm start`, `apm state`, `apm close`, or other commands

### Approach

`take.rs` lines 7–8 currently bail if `APM_AGENT_NAME` is unset. Replace that read with a call to `start::resolve_agent_name()`, which already implements the correct fallback chain (`APM_AGENT_NAME` → `$USER` → `$USERNAME` → `"apm"`).

Steps:
1. In `apm/src/cmd/take.rs`, replace:
   ```rust
   let new_agent = std::env::var("APM_AGENT_NAME")
       .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;
   ```
   with:
   ```rust
   let new_agent = super::start::resolve_agent_name();
   ```
2. Add an integration test in `apm/tests/integration.rs` that:
   - Creates a ticket in `in_design` state with a known agent
   - Unsets `APM_AGENT_NAME`, `USER`, and `USERNAME` then calls `take::run()`
   - Asserts the ticket's `agent` field equals `"apm"`
3. Add a second integration test variant with `USER` set to verify that path resolves correctly.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:41Z | — | new | philippepascal |
| 2026-03-30T16:37Z | new | in_design | philippepascal |
| 2026-03-30T16:39Z | in_design | specd | claude-0330-1640-b7f2 |
| 2026-03-30T17:01Z | specd | ready | philippepascal |
| 2026-03-30T17:02Z | ready | in_progress | philippepascal |
| 2026-03-30T17:05Z | in_progress | implemented | claude-0330-1702-b9d0 |
| 2026-03-30T17:09Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |