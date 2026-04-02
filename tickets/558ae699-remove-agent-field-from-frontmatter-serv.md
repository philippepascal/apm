+++
id = "558ae699"
title = "Remove agent field from frontmatter, server API, and apm list/show output"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "85311"
branch = "ticket/558ae699-remove-agent-field-from-frontmatter-serv"
created_at = "2026-04-02T20:53:58.923882Z"
updated_at = "2026-04-02T23:33:33.489950Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

Ticket frontmatter contains an `agent` field that records the name of the worker process currently handling the ticket. Workers are single-use processes that die when a ticket reaches `implemented`; the agent name has no durable value after that point. Despite this, the field propagates into three places:

1. **Frontmatter struct** — `Frontmatter.agent: Option<String>` is read and written via TOML. Dependent ticket 610be42e stops writing the field and changes CLI output; this ticket completes the cleanup by removing the struct field entirely and eliminating all server-side explicit references.
2. **Server API responses** — `WorkerInfo` in `apm-server/src/workers.rs` contains an explicit `agent: String` field populated from frontmatter and included in `GET /api/workers` JSON. The UI worker panel receives this but the field has no actionable meaning once a worker is not live.
3. **`take_ticket` handler** — `POST /api/tickets/:id/take` in `apm-server/src/main.rs` calls `handoff()` and has a fallback that writes `ticket.frontmatter.agent = Some(agent_name)` when no agent is set. After 610be42e updates `handoff()` to not require a prior agent value, this fallback becomes unreachable dead code; it still compiles and writes a stale value if ever hit.

The fix removes `agent` from the `Frontmatter` struct, the server worker response, and the `take_ticket` fallback — completing the cleanup started in 610be42e. Existing ticket files that still have `agent = "..."` in their TOML are unaffected: the `toml` crate silently ignores unknown fields when `deny_unknown_fields` is not set.

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
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:33Z | groomed | in_design | philippepascal |