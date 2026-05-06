+++
id = "42167022"
title = "Scaffold phi4-ollama direct wrapper"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/42167022-scaffold-phi4-ollama-direct-wrapper"
created_at = "2026-05-06T19:06:03.063876Z"
updated_at = "2026-05-06T20:48:29.208688Z"
+++

## Spec

### Problem

APM supports custom wrappers placed at `.apm/agents/<name>/wrapper.*`. When a ticket is dispatched with `agent = "phi4"`, APM invokes `.apm/agents/phi4/wrapper.*` instead of the built-in `claude` binary. No such directory exists yet, so Phi-4 (running locally via Ollama) cannot be used as a worker.

The wrapper must implement the full agentic loop itself: send the system prompt and user message to `http://localhost:11434/v1/chat/completions` with `model = "phi4"`, check the response for `tool_calls`, execute each tool locally, append tool results to the message history, and loop until the model stops issuing tool calls. Once done it emits canonical JSONL on stdout and calls `apm state` to transition the ticket. Phi-4's context window is 16 K tokens, so the worker instructions file must be concise enough to leave room for ticket content.

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
| 2026-05-06T19:06Z | — | new | philippepascal |
| 2026-05-06T20:47Z | new | groomed | philippepascal |
| 2026-05-06T20:48Z | groomed | in_design | philippepascal |