+++
id = "ba121f45"
title = "apm prompt command to deterministically build worker system prompt"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba121f45-apm-prompt-command-to-deterministically-"
created_at = "2026-05-14T21:14:16.356953Z"
updated_at = "2026-05-15T01:29:47.200209Z"
+++

## Spec

### Problem

Workers spawned via `apm start`, `apm work`, and the UI dispatch loop all call `resolve_system_prompt()` in `apm-core/src/start.rs`, which applies a 5-level priority cascade to produce the system prompt. The cascade is: (0) `transition.instructions`, (1) profile.instructions, (2) workers.instructions, (3) `.apm/agents/<agent>/apm.<role>.md`, (4) built-in default. Because Level 0 always wins when a transition has `instructions` set — and nearly every transition in `workflow.toml` does — the per-agent files at Level 3 are unreachable dead code. A `pi`-agent transition still gets the `default` spec-writer prompt even if `.apm/agents/pi/apm.spec-writer.md` exists. There is also no CLI surface to inspect what any given (agent, role, ticket) tuple would receive; the only way to see the real prompt is to launch a live worker.\n\nAdd an `apm prompt <ticket-id>` command that deterministically assembles and prints the system prompt for a ticket's current state without spawning a worker. Promote the per-agent file lookup to the highest-priority level so agent-specific overrides actually take effect. Consolidate the three spawn paths — `run()`, `run_next()`, and `spawn_next_worker()` — onto a single `build_system_prompt()` function so the CLI output is guaranteed identical to what a worker receives.

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
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-14T21:22Z | groomed | in_design | philippe |
| 2026-05-15T01:21Z | in_design | groomed | philippe |
| 2026-05-15T01:29Z | groomed | in_design | philippe |