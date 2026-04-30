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

A ticket can override which agent handles its workers, either across all phases or per profile. Useful for: debugging a stuck ticket with mock-happy, mixing agents per phase (Claude for spec, Codex for impl), forcing a specific agent for a regression test.

**Reference spec:** `docs/agent-wrappers.md` — section 'Frontmatter override'.

**Scope:**
- `apm-core/src/ticket/ticket_fmt.rs` (`Frontmatter` struct):
  - Add `pub agent: Option<String>` — single all-profiles override
  - Add `pub agent_overrides: HashMap<String, String>` — per-profile map (profile name → agent name)
  - Both are `#[serde(default)]` and skip-serializing-if-empty.
- `apm-core/src/start.rs`:
  - Update agent resolution chain to (per spawn, where P = worker profile):
    1. `frontmatter.agent_overrides[P]` if present
    2. `frontmatter.agent` if present
    3. `[worker_profiles.<P>].agent` if set
    4. `[workers].agent`
  - Resolution happens at spawn time, reading the ticket's current frontmatter from its branch.
- `apm validate`:
  - For each ticket whose `frontmatter.agent` or any value in `agent_overrides` is set, the named agent must resolve to a built-in or project script. Report missing agents as ticket-level errors with the ticket id and the offending agent name.
- Document the override fields in `apm.spec-writer.md` and `apm.worker.md` so agents know they exist (briefly — ticket-frontmatter overrides are a supervisor tool, not an agent tool).

**Out of scope:**
- Per-transition agent mapping (a `{transition: agent}` map). Spec defers this as a v2 contract extension; not in v1.
- A CLI command to set the override (`apm set <id> agent X`). Could be added to the existing `apm set` field list, but is a small follow-up. Note in spec.
- Surfacing the override in `apm show` output. Could be a small follow-up.

**Tests:**
- Resolution test: each level wins over the next.
- Validate test: ticket with `agent = "unknown-wrapper"` produces a clear error.
- Round-trip test: frontmatter with both fields serializes cleanly and parses back identically.

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
