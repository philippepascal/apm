+++
id = 61
title = "Create apm.spec-writer.md: instruction file for spec-writing agents"
state = "closed"
priority = 2
effort = 2
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1430-main"
branch = "ticket/0061-create-apm-spec-writer-md-instruction-fi"
created_at = "2026-03-29T19:12:34.623619Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

Spec-writing agents (those picking up `new` or `ammend` tickets) currently receive only the generic `apm.agents.md` instruction file. There is no dedicated guidance for writing good specs in this project — what quality bar to meet, how to structure each section, how to evaluate effort/risk, etc.

The spec quality bar documented in `apm.agents.md` is brief. A richer, dedicated instruction file would give spec-writing agents more context and reduce the need for amendment cycles.

### Acceptance criteria

- [x] A file `apm.spec-writer.md` exists at the repo root with practical guidance for spec-writing agents (problem framing, acceptance criteria quality, approach detail, effort/risk calibration)
- [x] `apm.toml` references `apm.spec-writer.md` as the `instructions` for the `new` and `ammend` states via a new `instructions` field on those state entries
- [x] `apm-core` config parsing accepts an optional `instructions: String` field on `StateConfig` without breaking existing configs that omit it
- [x] `apm show <id>` or `apm next --json` does not need to change — the instructions field is advisory metadata for agent tooling, not enforced at runtime
- [x] The content of `apm.spec-writer.md` covers: problem framing, acceptance criteria format, out-of-scope discipline, approach depth, effort/risk scale, and the spec quality bar from `apm.agents.md`

### Out of scope

- Runtime enforcement: `apm state <id> specd` does not validate that instructions were followed
- Changing the agent startup flow to auto-load instructions
- Creating a separate `apm spec` command
- Any changes to `apm.worker.md` (covered by ticket #62)

### Approach

1. Add `instructions: Option<String>` to `StateConfig` in `apm-core/src/config.rs`. Mark it `#[serde(default)]`.

2. Add `instructions = "apm.spec-writer.md"` to the `new` and `ammend` state entries in `apm.toml`.

3. Write `apm.spec-writer.md` at the repo root with:
   - When this file applies (picking up `new` or `ammend` tickets)
   - Problem section: what makes a good problem statement
   - Acceptance criteria: one criterion = one independently testable behaviour, checkbox format
   - Out of scope: explicit, not implied — list things that could be mistaken for in-scope
   - Approach: enough detail that an implementer can follow without reading the problem again
   - Effort scale: 1 = trivial (< 1 hour), 5 = medium (half day), 10 = large (multi-day)
   - Risk scale: 1 = no unknowns, 5 = some uncertainty, 10 = high uncertainty or blast radius

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:57Z | new | in_design | claude-spec-61 |
| 2026-03-29T23:09Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:18Z | specd | ready | apm |
| 2026-03-29T23:37Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:39Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |