+++
id = "0ca3e019"
title = "Frontmatter agent + agent_overrides for per-ticket wrapper choice"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0ca3e019-frontmatter-agent-agent-overrides-for-pe"
created_at = "2026-04-30T20:03:58.532325Z"
updated_at = "2026-04-30T21:50:38.458676Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "6cac8518"]
+++

## Spec

### Problem

The agent-selection config introduced by tickets d3b93b95 and 6cac8518 operates at the project level: every ticket is dispatched to whatever agent the `[workers]` block or the matching `[worker_profiles.<P>]` block names. There is no per-ticket escape hatch. A supervisor who wants to debug a specific stuck ticket with `mock-happy`, force a regression ticket to use a particular agent, or give one unusual ticket a per-phase agent mix must edit `.apm/config.toml`, run, then revert — a fragile workflow that also affects all concurrently running workers.

This ticket adds two optional fields to ticket frontmatter that let a supervisor override agent selection for a single ticket, narrowly, without touching shared config:

- `agent = "<name>"` — every worker spawn for this ticket uses the named agent, regardless of which profile the transition selects.
- `[agent_overrides]` table — per-profile selection (`spec_agent = "claude"`, `impl_agent = "mock-random"`), so different phases of the same ticket can use different agents.

Both fields are optional and additive. Tickets that set neither field are unchanged. The fields override the config chain but affect only the one ticket where they appear.

The full resolution order (per spawn, where P is the profile name declared by the triggering transition):
1. `frontmatter.agent_overrides[P]` if present
2. `frontmatter.agent` if present
3. `[worker_profiles.<P>].agent` from config (ticket 6cac8518)
4. `[workers].agent` global default (ticket 6cac8518)

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
| 2026-04-30T21:50Z | groomed | in_design | philippepascal |