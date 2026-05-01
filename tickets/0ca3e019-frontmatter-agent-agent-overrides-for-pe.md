+++
id = "0ca3e019"
title = "Frontmatter agent + agent_overrides for per-ticket wrapper choice"
state = "specd"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0ca3e019-frontmatter-agent-agent-overrides-for-pe"
created_at = "2026-04-30T20:03:58.532325Z"
updated_at = "2026-04-30T21:57:22.347867Z"
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

**Files changed:** `ticket_fmt.rs`, `start.rs`, `validate.rs`, `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`.

#### `apm-core/src/ticket/ticket_fmt.rs` — `Frontmatter` struct

Add `use std::collections::HashMap;` to imports (`indexmap::IndexMap` is already there but `std::collections::HashMap` is not).

Add two fields to `Frontmatter` after `depends_on`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub agent: Option<String>,

#[serde(default, skip_serializing_if = "HashMap::is_empty")]
pub agent_overrides: HashMap<String, String>,
```

`agent_overrides` serializes as a TOML inline table when non-empty. Verify via the round-trip test that both the inline form and the subtable form parse correctly — no special serde attribute needed. `JsonSchema` is already derived; `HashMap<String, String>` is compatible by default.

#### `apm-core/src/start.rs` — Agent resolution

Add a free function after `effective_spawn_params`:

```rust
fn apply_frontmatter_agent(
    agent: &mut String,
    frontmatter: &ticket_fmt::Frontmatter,
    profile_name: &str,
) {
    if let Some(ov) = frontmatter.agent_overrides.get(profile_name) {
        *agent = ov.clone();
    } else if let Some(a) = &frontmatter.agent {
        *agent = a.clone();
    }
    // else: keep config-resolved agent unchanged
}
```

Call this in each of the three spawn paths — `run()`, `run_next()`, `spawn_next_worker()` — immediately after `effective_spawn_params()` returns and before `WrapperContext` is constructed (introduced by d3b93b95). `ticket.frontmatter` is already loaded at all three call sites. The profile name is already available from `resolve_profile()`. `EffectiveWorkerParams.agent` (added by 6cac8518) is the field being mutated.

#### `apm-core/src/validate.rs` — `verify_tickets`

In the per-ticket loop inside `verify_tickets()`, after existing checks, collect all agent names declared in frontmatter and check each against `wrapper::resolve_builtin()` (introduced by d3b93b95):

```rust
let agents_to_check: Vec<&str> = ticket.frontmatter.agent
    .as_deref()
    .into_iter()
    .chain(ticket.frontmatter.agent_overrides.values().map(String::as_str))
    .collect();

for name in agents_to_check {
    if wrapper::resolve_builtin(name).is_none() {
        errors.push(format!(
            "ticket {}: agent {:?} is not a known built-in",
            ticket.frontmatter.id, name
        ));
    }
}
```

This checks only built-ins. When ticket 2c32a282 lands and `resolve_wrapper()` subsumes `resolve_builtin()`, this call site can be upgraded in that ticket.

#### `.apm/apm.spec-writer.md` and `.apm/apm.worker.md`

Append a short note near the end of each file:

> **Frontmatter agent override** (supervisor tool): A supervisor may add `agent = "<name>"` or an `[agent_overrides]` table to a ticket's frontmatter to select a specific agent for that ticket or for individual profiles. Do not set these fields yourself — they are a supervisor-level escape hatch for debugging or per-ticket specialisation.

#### Tests

`apm-core/src/ticket/ticket_fmt.rs` — add to the existing test module:
- `frontmatter_agent_round_trip`: parse TOML with `agent = "mock-happy"` and two `agent_overrides` entries; serialize; re-parse; assert fields match.
- `frontmatter_agent_omitted_when_unset`: parse minimal frontmatter; serialize; assert neither `agent` nor `agent_overrides` appears in output.

`apm-core/src/start.rs` — add to existing test module:
- `apply_fm_profile_override_wins`: `agent_overrides = {impl_agent: "mock-happy"}`, `agent = "mock-sad"`, profile `"impl_agent"` → `"mock-happy"`.
- `apply_fm_agent_field_wins_when_no_profile_match`: `agent_overrides = {}`, `agent = Some("mock-sad")`, profile `"impl_agent"` → `"mock-sad"`.
- `apply_fm_profile_override_beats_agent_field`: `agent_overrides = {impl_agent: "claude"}`, `agent = Some("mock-random")`, profile `"impl_agent"` → `"claude"`.
- `apply_fm_no_fields_unchanged`: both fields empty/None, starting agent `"claude"` → unchanged.

`apm-core/src/validate.rs` — add to existing test module:
- `validate_unknown_frontmatter_agent_is_error`: ticket with `agent = "nonexistent-bot"`; assert error contains ticket id and agent name.
- `validate_unknown_agent_in_overrides_is_error`: ticket with `agent_overrides` value `"nonexistent-bot"`; assert error contains ticket id and agent name.
- `validate_known_frontmatter_agent_passes`: ticket with `agent = "claude"`; assert no errors from the agent check.

### 1. `apm-core/src/ticket/ticket_fmt.rs` — `Frontmatter` struct

Add `use std::collections::HashMap;` to the imports (currently not imported; `indexmap::IndexMap` is the only map in scope).

Add two fields to `Frontmatter` after `depends_on`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub agent: Option<String>,

#[serde(default, skip_serializing_if = "HashMap::is_empty")]
pub agent_overrides: HashMap<String, String>,
```

`agent_overrides` serializes as a TOML inline table when non-empty. Verify via the round-trip test that both the inline form (`agent_overrides = { spec_agent = "claude" }`) and the subtable form (`[agent_overrides]`) parse correctly; no special serde attribute is needed either way.

`JsonSchema` is already derived on `Frontmatter`; `HashMap<String, String>` is schema-compatible by default.

---

### 2. `apm-core/src/start.rs` — Agent resolution

Add a free function after `effective_spawn_params`:

```rust
fn apply_frontmatter_agent(
    agent: &mut String,
    frontmatter: &ticket_fmt::Frontmatter,
    profile_name: &str,
) {
    if let Some(override_agent) = frontmatter.agent_overrides.get(profile_name) {
        *agent = override_agent.clone();
    } else if let Some(a) = &frontmatter.agent {
        *agent = a.clone();
    }
    // else: keep config-resolved agent unchanged
}
```

Call this in each of the three spawn paths — `run()`, `run_next()`, and `spawn_next_worker()` — immediately after `effective_spawn_params()` returns and before `WrapperContext` is constructed (introduced by d3b93b95). The ticket's frontmatter (`ticket.frontmatter`) is already loaded at all three call sites. The profile name is the string resolved by `resolve_profile()`.

`EffectiveWorkerParams.agent` (added by 6cac8518) is the field being mutated. After d3b93b95 + 6cac8518 land, `WrapperContext` is constructed from `EffectiveWorkerParams`; the agent name flows through unchanged.

---

### 3. `apm-core/src/validate.rs` — `verify_tickets`

In the per-ticket loop inside `verify_tickets()`, after existing checks, collect all agent names declared in frontmatter:

```rust
let agents_to_check: Vec<&str> = ticket.frontmatter.agent
    .as_deref()
    .into_iter()
    .chain(ticket.frontmatter.agent_overrides.values().map(String::as_str))
    .collect();

for name in agents_to_check {
    if wrapper::resolve_builtin(name).is_none() {
        errors.push(format!(
            "ticket {}: agent {:?} is not a known built-in",
            ticket.frontmatter.id, name
        ));
    }
}
```

Import `crate::wrapper` (introduced by d3b93b95). This check covers only built-ins; custom project scripts from ticket 2c32a282 are not checked here — when 2c32a282 lands and `resolve_wrapper()` subsumes `resolve_builtin()`, this call site can be upgraded in that ticket.

---

### 4. `.apm/apm.spec-writer.md` and `.apm/apm.worker.md`

Append a short note to each file near the end:

> **Frontmatter agent override** (supervisor tool): A supervisor may add `agent = "<name>"` or an `[agent_overrides]` table to a ticket's frontmatter to select a specific agent for that ticket or for individual profiles. Do not set these fields yourself — they are a supervisor-level escape hatch for debugging or per-ticket specialisation.

---

### 5. Tests

**`apm-core/src/ticket/ticket_fmt.rs`** — add to the existing test module:

- `frontmatter_agent_round_trip`: parse TOML with `agent = "mock-happy"` and two entries in `agent_overrides`; serialize; re-parse; assert both fields match original.
- `frontmatter_agent_omitted_when_unset`: parse minimal frontmatter without `agent` or `agent_overrides`; serialize; assert neither field appears in the serialized string.

**`apm-core/src/start.rs`** — add to the existing test module:

- `apply_fm_profile_override_wins`: `agent_overrides = {impl_agent: "mock-happy"}`, `agent = "mock-sad"`, profile = `"impl_agent"` → result is `"mock-happy"`.
- `apply_fm_agent_field_wins_when_no_profile_match`: `agent_overrides = {}`, `agent = Some("mock-sad")`, profile = `"impl_agent"` → result is `"mock-sad"`.
- `apply_fm_profile_override_beats_agent_field`: `agent_overrides = {impl_agent: "claude"}`, `agent = Some("mock-random")`, profile = `"impl_agent"` → result is `"claude"`.
- `apply_fm_no_fields_unchanged`: both fields empty/None, starting agent = `"claude"` → result unchanged.

**`apm-core/src/validate.rs`** — add to the existing test module:

- `validate_unknown_frontmatter_agent_is_error`: ticket with `agent = "nonexistent-bot"`; assert error contains ticket id and `"nonexistent-bot"`.
- `validate_unknown_agent_in_overrides_is_error`: ticket with `agent_overrides` value `"nonexistent-bot"`; assert error contains ticket id and `"nonexistent-bot"`.
- `validate_known_frontmatter_agent_passes`: ticket with `agent = "claude"`; assert no errors from the agent check.

### Open questions


### Amendment requests

- [ ] Add a cross-ticket TODO note in the Approach: this ticket's validate check uses `wrapper::resolve_builtin(name)` (correct as of foundation ticket d3b93b95). Once ticket 2c32a282 (custom wrappers) lands, that call site must be upgraded to `wrapper::resolve_wrapper(root, name)` so frontmatter agent overrides referencing project-defined custom wrappers also validate correctly. Without this note, the upgrade is invisible and a future ticket either misses it or has to re-derive the dependency. Adding it as an explicit AC ("After 2c32a282 lands, validate's frontmatter-agent check uses resolve_wrapper, not resolve_builtin") closes the loop.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:50Z | groomed | in_design | philippepascal |
| 2026-04-30T21:57Z | in_design | specd | claude-0430-2150-ea08 |