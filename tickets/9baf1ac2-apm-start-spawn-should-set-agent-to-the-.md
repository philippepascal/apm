+++
id = "9baf1ac2"
title = "apm start --spawn should set agent to the worker's name, not the delegator's"
state = "new"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
branch = "ticket/9baf1ac2-apm-start-spawn-should-set-agent-to-the-"
created_at = "2026-03-30T05:56:35.911177Z"
updated_at = "2026-03-30T05:56:35.911177Z"
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

How the implementation will work.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T05:56Z | — | new | claude-0330-0245-main |
