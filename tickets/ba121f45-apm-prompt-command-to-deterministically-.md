+++
id = "ba121f45"
title = "apm prompt command to deterministically build worker system prompt"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba121f45-apm-prompt-command-to-deterministically-"
created_at = "2026-05-14T21:14:16.356953Z"
updated_at = "2026-05-14T21:14:16.356953Z"
+++

## Spec

### Problem

Workers spawned via `apm start`, `apm work`, and the UI dispatch loop currently get their system prompt from `resolve_system_prompt` in `apm-core/src/start.rs` — a 5-level priority cascade. The output of that function is the entire system prompt. There's no command-line way to inspect or test what a worker will actually receive before it spawns, and the assembly is duplicated/implicit across the three spawn paths.

This makes debugging prompt issues hard. Recent example: pi-worker received the default spec-writer prompt (transition.instructions wins), but the prompt told it to 'read apm.agents.md' — a filename that doesn't exist (only `agents.md` does), and pi can't reliably fetch external files anyway. Without a way to print the actual assembled prompt, that mismatch was only visible by running a real worker.

The user also wants per-agent files (e.g. `.apm/agents/pi/apm.spec-writer.md`) to override the default when present. Right now they exist as dead documentation because Level 0 (transition.instructions) always wins.

Add a new `apm prompt` (or `apm agent prompt`, name TBD in spec) command that deterministically assembles the system prompt for a given (agent, role, ticket-id) tuple and prints it. `apm start`, `apm work`, and the UI must internally call this same code path so what you see is what the worker sees.

Acceptance: a single function builds the prompt; CLI exposes it; spawn paths consume it; per-agent files override the defaults when present; existing transitions in workflow.toml continue to work as a fallback.

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
