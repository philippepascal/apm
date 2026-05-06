+++
id = "42167022"
title = "Scaffold phi4-ollama direct wrapper"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/42167022-scaffold-phi4-ollama-direct-wrapper"
created_at = "2026-05-06T19:06:03.063876Z"
updated_at = "2026-05-06T20:47:32.057298Z"
+++

## Spec

### Problem

Scaffold .apm/agents/phi4/ — a custom APM wrapper that calls Ollama's OpenAI-compatible API directly (http://localhost:11434/v1/chat/completions) with Phi-4 as the model. The wrapper must implement the full agentic tool-call loop in the script itself (no framework): send system prompt + user message, check for tool_calls in the response, execute tools locally (bash, read_file, write_file, str_replace), append tool results, loop until no more tool calls, then emit canonical JSONL and call apm state. Files to create: .apm/agents/phi4/wrapper.sh (the loop, preferably Python for JSON handling), .apm/agents/phi4/manifest.toml (contract_version=1, parser=canonical), .apm/agents/phi4/apm.worker.md (augmented worker instructions — the standard apm.worker.md content plus a tools section explaining bash/read_file/write_file/str_replace function calling, autonomous operation rules, and reminder to call apm state at end). All files are new additions; nothing existing is modified. Phi-4 context window is 16K so the worker instructions must be concise.

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
