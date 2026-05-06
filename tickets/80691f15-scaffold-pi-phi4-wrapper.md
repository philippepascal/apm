+++
id = "80691f15"
title = "Scaffold pi-phi4 wrapper"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/80691f15-scaffold-pi-phi4-wrapper"
created_at = "2026-05-06T19:06:14.397074Z"
updated_at = "2026-05-06T21:28:36.987169Z"
+++

## Spec

### Problem

APM has no wrapper for the pi CLI (https://pi.dev), so projects that want to run Phi-4 via Ollama as the worker model cannot integrate with APM's agent dispatch system. The pi CLI provides a clean interface to locally-hosted models, but APM requires a four-file bridge — `manifest.toml`, `wrapper.sh`, `parser.py`, and `apm.worker.md` — before it can spawn pi as a worker.

This ticket creates those four files under `.apm/agents/pi/`. The wrapper invokes `pi --mode json --provider ollama --model phi4` (model overridable via `APM_OPT_MODEL`), the parser translates pi's JSONL event stream to APM canonical events, the manifest declares the parser contract, and the worker instructions tell the pi agent how to work a ticket. No existing file is changed.

### Acceptance criteria

- [ ] `.apm/agents/pi/manifest.toml` exists with `contract_version = 1`, `parser = "external"`, and `parser_command = "./parser.py"`
- [ ] `.apm/agents/pi/wrapper.sh` is executable and invokes `pi --mode json --provider ollama --model <model>` where `<model>` is `$APM_OPT_MODEL` when set, otherwise `phi4`
- [ ] `wrapper.sh` constructs the prompt by combining the contents of `APM_SYSTEM_PROMPT_FILE` and `APM_USER_MESSAGE_FILE` (system first, then user message)
- [ ] `wrapper.sh` includes a comment block documenting how to configure Ollama as a provider in `~/.pi/agent/models.json`
- [ ] `.apm/agents/pi/parser.py` reads pi's JSONL from stdin and emits at least one `{"type": "text", "text": "..."}` line containing the assistant's response text
- [ ] `parser.py` emits `{"type": "result", "text": ""}` when pi's `agent_end` event is received
- [ ] `parser.py` silently skips all pi event types that do not carry assistant text (no output, no error)
- [ ] `.apm/agents/pi/apm.worker.md` exists with worker instructions scoped to pi's capabilities (no Claude-specific flags, no tool-augmentation section)
- [ ] No existing file under `.apm/agents/` or anywhere else in the repo is modified

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
| 2026-05-06T21:28Z | groomed | in_design | philippepascal |