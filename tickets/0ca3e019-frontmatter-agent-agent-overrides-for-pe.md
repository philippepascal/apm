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

- [ ] `Frontmatter` struct has `pub agent: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
- [ ] `Frontmatter` struct has `pub agent_overrides: HashMap<String, String>` with `#[serde(default, skip_serializing_if = "HashMap::is_empty")]`
- [ ] A ticket frontmatter containing `agent = "mock-happy"` round-trips through TOML serialize → parse → serialize without loss
- [ ] A ticket frontmatter containing `[agent_overrides]` round-trips through TOML serialize → parse → serialize without loss
- [ ] A ticket with neither `agent` nor `agent_overrides` set serializes without either field appearing in the output
- [ ] When spawning a worker for profile `P` and `frontmatter.agent_overrides[P]` is set, that value is used as the agent name
- [ ] When `agent_overrides` has no entry for profile `P` but `frontmatter.agent` is set, `frontmatter.agent` is used
- [ ] When `frontmatter.agent_overrides[P]` is set and `frontmatter.agent` is also set, the profile-specific override wins
- [ ] When neither frontmatter field is set, the config-resolved agent (from 6cac8518) is used unchanged
- [ ] `apm validate` reports an error for a ticket whose `frontmatter.agent` names a non-existent built-in; the error message includes the ticket id
- [ ] `apm validate` reports an error for a ticket whose `frontmatter.agent_overrides` contains a value naming a non-existent built-in; the error message includes the ticket id and the offending agent name
- [ ] `apm validate` does not report an error for a ticket whose `frontmatter.agent` is `"claude"`
- [ ] `.apm/apm.spec-writer.md` contains a brief note that supervisors may set `agent` or `[agent_overrides]` in frontmatter
- [ ] `.apm/apm.worker.md` contains a brief note that supervisors may set `agent` or `[agent_overrides]` in frontmatter

### Out of scope

- Per-transition agent mapping (a `{transition_name: agent}` map). The design doc explicitly defers this as a v2 contract extension; per-profile granularity is sufficient for v1.
- A CLI command to set the override (e.g. `apm set <id> agent <name>`). Could be added to `apm set`'s field list; noted as a small follow-up, not in this ticket.
- Surfacing the override in `apm show` output. The fields are present in frontmatter and visible via `cat`; display in `apm show` is a follow-up.
- Validating frontmatter agent names against custom project wrappers in `.apm/agents/<name>/`. `apm validate` in this ticket only checks built-ins via `resolve_builtin()`. Custom-wrapper validation is ticket 2c32a282's territory.
- Any changes to `apm set`, `apm show`, or `apm agents` subcommands.

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