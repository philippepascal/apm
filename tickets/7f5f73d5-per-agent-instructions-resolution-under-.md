+++
id = "7f5f73d5"
title = "Per-agent instructions resolution under .apm/agents/<name>/"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7f5f73d5-per-agent-instructions-resolution-under-"
created_at = "2026-04-30T20:03:33.687625Z"
updated_at = "2026-04-30T21:42:25.580070Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

The current `resolve_system_prompt` function in `apm-core/src/start.rs` uses a flat 4-level chain that ends in a silent hardcoded fallback string (`"You are an APM worker agent."`). It has no concept of which agent is being run and resolves the system prompt from a single flat path — `.apm/apm.worker.md` — shared across all agents. As custom wrappers are introduced (ticket 2c32a282), different agents may need different prompt conventions (Codex structured tags, Aider concise context, etc.); a single flat `.apm/apm.worker.md` cannot express per-agent defaults.

The desired behaviour is a 4-level resolution chain per spawn (agent A, role = worker|spec-writer, profile P):
1. `[worker_profiles.<P>].instructions` — project-level per-profile override
2. `[workers].instructions` — project-level global override, applies to all profiles
3. `.apm/agents/<A>/apm.<role>.md` — project-supplied per-agent file, if it exists
4. APM's bundled default for agent A (via `include_str!`, built-in agents only)
5. Hard error if none of the above resolve

Existing projects keep working without edits because their `[worker_profiles.spec_agent] instructions = ".apm/apm.spec-writer.md"` and `[worker_profiles.impl_agent] instructions = ".apm/apm.worker.md"` satisfy level 1. No migration is required.

The silent hardcoded fallback and the `StateConfig.instructions`-as-system-prompt path are both removed. `StateConfig.instructions` is a per-state annotation used for display and tooling (the field remains on the struct) but is no longer consumed by `resolve_system_prompt`.

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
| 2026-04-30T21:42Z | groomed | in_design | philippepascal |