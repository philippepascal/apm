+++
id = "9baf1ac2"
title = "apm start --spawn should set agent to the worker's name, not the delegator's"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/9baf1ac2-apm-start-spawn-should-set-agent-to-the-"
created_at = "2026-03-30T05:56:35.911177Z"
updated_at = "2026-03-30T06:16:44.728622Z"
+++

## Spec

### Problem

When `apm start --spawn` claims a ticket, it sets `agent` to the delegator's
`APM_AGENT_NAME`. The spawned worker runs under its own agent name (visible in
the worker log as `Agent name: claude-MMDD-HHMM-XXXX`) but the ticket
frontmatter is never updated to reflect that. As a result, `apm list` shows all
spawned tickets as owned by the delegator, making it impossible to tell which
worker is handling which ticket.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

**Use PID as the worker's agent identifier, not a generated name.**

Workers are short-lived single-ticket processes. PID is already unique per
process, directly usable with `kill`, and consistent with the `.apm-worker.pid`
files planned in ticket #84. Delegators (interactive sessions) keep
`APM_AGENT_NAME` — they are long-lived and supervisor-visible.

**Claim/spawn ordering change in `apm start --spawn` (`start.rs`):**

Currently the ticket is claimed (agent = delegator's `APM_AGENT_NAME`) before
the process is spawned. To write the worker's PID instead:

1. Claim the ticket with a placeholder (or skip the agent field until step 3)
2. Spawn the child process
3. Get the PID back from the `Child` handle
4. Update the ticket's `agent` field to the PID string and commit

**Compatibility analysis:**

- `apm next` / `apm start --next` filtering — safe; agent value is not used for
  filtering (removed in a prior fix)
- Startup resume check (`apm list --state in_progress` matching agent name) —
  safe; workers receive their ticket ID directly in the spawn prompt and do not
  rely on agent-name lookup to resume
- `apm take` — unaffected
- Ticket #84 (`apm workers`) — being implemented concurrently; its `.apm-worker.pid`
  file approach is consistent with PID-as-agent. Coordinate to avoid conflicts.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T05:56Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:16Z | new | in_design | claude-0330-0245-main |
