+++
id = "80691f15"
title = "Scaffold pi-phi4 wrapper"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/80691f15-scaffold-pi-phi4-wrapper"
created_at = "2026-05-06T19:06:14.397074Z"
updated_at = "2026-05-06T22:22:19.247837Z"
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

- Installing the pi CLI or Ollama — must be set up separately by the user
- Writing or modifying `~/.pi/agent/models.json` — only documented in wrapper comments
- Handling pi tool calls in the parser — pi executes tools internally; the parser only needs to surface text output
- Supporting non-Ollama providers (e.g. OpenAI, Anthropic via pi)
- Integration testing against a live pi/Ollama installation
- Registering `pi` as an active agent in `.apm/config.toml` — that is a project-level configuration decision

### Approach

All four files are new additions to `.apm/agents/pi/`. No existing file is touched.

#### manifest.toml

Create `.apm/agents/pi/manifest.toml`:

```toml
contract_version = 1
parser = "external"
parser_command = "./parser.py"
```

#### wrapper.sh

Create `.apm/agents/pi/wrapper.sh` and `chmod +x` it.

The script reads both prompt files, combines them into a single string (system prompt first, a separator line, then user message), and passes the result as the positional `<prompt>` argument to pi. If pi gains a `--system` flag in a future version, the script can be updated to pass them separately; for now concatenation is the safe baseline.

```sh
#!/bin/sh
# APM pi wrapper — invokes pi CLI with Phi-4 via Ollama.
#
# Prerequisites:
#   1. Install pi CLI: see https://pi.dev/docs/install
#   2. Install Ollama: see https://ollama.com
#   3. Pull the model: ollama pull phi4
#   4. Configure ~/.pi/agent/models.json to register the Ollama provider:
#
#      {
#        "ollama": {
#          "type": "ollama",
#          "base_url": "http://localhost:11434",
#          "models": {
#            "phi4": { "context_length": 16384 }
#          }
#        }
#      }
#
#   Adjust base_url and context_length to match your Ollama installation.
set -e

model="${APM_OPT_MODEL:-phi4}"
sys=$(cat "$APM_SYSTEM_PROMPT_FILE")
msg=$(cat "$APM_USER_MESSAGE_FILE")

exec pi --mode json --provider ollama --model "$model" "$sys

---

$msg"
```

Note: `exec` replaces the shell process, so the parser receives pi's stdout directly. No `apm state` call is needed in the wrapper — APM drives state transitions based on parsed output.

#### parser.py

Create `.apm/agents/pi/parser.py` and `chmod +x` it.

Pi emits JSONL where each line is an `AgentSessionEvent` object. The events relevant for APM are:

| pi event type | `assistantMessageEvent.type` | Action |
|---|---|---|
| `message_end` | — | emit `{"type": "text", "text": "<full response>"}` |
| `agent_end` | — | emit `{"type": "result", "text": ""}` then exit |
| anything else | — | skip silently |

Use `message_end` (not streaming `text_delta`) for simplicity: the `message` field on `message_end` contains the final `AssistantMessage` with a `content` array of text blocks. Join all text-block strings and emit as one APM text event.

```python
#!/usr/bin/env python3
import sys
import json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        continue

    t = event.get("type")

    if t == "message_end":
        msg = event.get("message", {})
        parts = [
            block.get("text", "")
            for block in msg.get("content", [])
            if block.get("type") == "text"
        ]
        text = "".join(parts)
        if text:
            print(json.dumps({"type": "text", "text": text}), flush=True)

    elif t == "agent_end":
        print(json.dumps({"type": "result", "text": ""}), flush=True)
        break
```

If the exact field paths differ from what pi actually emits (verify with `pi --mode json --provider ollama --model phi4 "hello" 2>&1 | head -20`), adjust the `content` traversal accordingly. The block structure `[{"type": "text", "text": "..."}]` mirrors what other pi-compatible parsers use.

#### apm.worker.md

Create `.apm/agents/pi/apm.worker.md` as a copy of `.apm/agents/default/apm.worker.md` with these changes:

- Remove the `Shell discipline` section (pi enforces its own execution model)
- Remove the `Path discipline` section (pi manages its own filesystem access)
- Replace the `Tests` section header and body with: "Run the project's test suite according to `## Spec → Approach` in your ticket. All tests must pass before calling `apm state <id> implemented`."
- Replace all references to `claude` binary with `pi`
- Keep `Scope limits`, `Before writing any code`, `Minimal-change discipline`, `Commit format`, `Finishing implementation`, `Side tickets`, and `Blocked state` sections unchanged

The resulting file teaches the pi agent what APM commands to run and what constraints apply, without referencing Claude-specific flags or tool augmentation.

#### Verification

After creating all four files, run:

```sh
ls -la .apm/agents/pi/
```

Confirm all four files exist and `wrapper.sh` and `parser.py` have the executable bit set.

### Open questions


### Amendment requests

- [ ] wrapper.sh: replace 'exec pi' with plain invocation followed by 'apm state $APM_TICKET_ID implemented || true' — exec prevents the shell from calling apm state if phi4 lacks bash tool access
- [ ] apm.worker.md: explicitly instruct the agent that its final action must be calling bash("apm state $APM_TICKET_ID implemented") — phi4 calling it via tool is the primary path; shell script is the fallback

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-06T19:06Z | — | new | philippepascal |
| 2026-05-06T20:47Z | new | groomed | philippepascal |
| 2026-05-06T21:28Z | groomed | in_design | philippepascal |
| 2026-05-06T21:33Z | in_design | specd | claude-0506-2128-c940 |
| 2026-05-06T22:22Z | specd | ammend | philippepascal |
| 2026-05-06T22:22Z | ammend | in_design | philippepascal |
