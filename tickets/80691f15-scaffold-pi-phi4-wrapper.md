+++
id = "80691f15"
title = "Scaffold pi-phi4 wrapper"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/80691f15-scaffold-pi-phi4-wrapper"
created_at = "2026-05-06T19:06:14.397074Z"
updated_at = "2026-05-06T20:47:58.424621Z"
+++

## Spec

### Problem

Scaffold .apm/agents/pi/ — an APM wrapper that delegates to the pi CLI (https://pi.dev) configured to use Phi-4 via Ollama. Pi supports --mode json for a JSONL event stream and --print for simple non-interactive use. Pi has --provider and --model flags. For Ollama: provider config lives in ~/.pi/agent/models.json; invoke with: pi --mode json --provider ollama --model phi4 <prompt>. Files to create: .apm/agents/pi/wrapper.sh (invokes pi CLI with --mode json, reads APM_SYSTEM_PROMPT_FILE and APM_USER_MESSAGE_FILE, uses APM_OPT_MODEL defaulting to phi4, calls apm state at end), .apm/agents/pi/manifest.toml (contract_version=1, parser=external, parser_command=./parser.py), .apm/agents/pi/parser.py (reads pi's --mode json JSONL from stdin, translates to APM canonical {type, text} events on stdout — need to check pi's actual JSON event schema from its docs/json.md), .apm/agents/pi/apm.worker.md (standard worker instructions; pi handles tool execution internally so no tool augmentation needed). Also document in the wrapper: how to configure Ollama as a provider in ~/.pi/agent/models.json. All files are new additions; nothing existing is modified. The pi CLI must be installed separately (not in scope of this ticket).

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
