+++
id = "42167022"
title = "Scaffold phi4-ollama direct wrapper"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/42167022-scaffold-phi4-ollama-direct-wrapper"
created_at = "2026-05-06T19:06:03.063876Z"
updated_at = "2026-05-06T23:39:20.246684Z"
+++

## Spec

### Problem

APM supports custom wrappers placed at `.apm/agents/<name>/wrapper.*`. When a ticket is dispatched with `agent = "phi4"`, APM invokes `.apm/agents/phi4/wrapper.*` instead of the built-in `claude` binary. No such directory exists yet, so Phi-4 (running locally via Ollama) cannot be used as a worker.

The wrapper must implement the full agentic loop itself: send the system prompt and user message to `http://localhost:11434/v1/chat/completions` with `model = "phi4"`, check the response for `tool_calls`, execute each tool locally, append tool results to the message history, and loop until the model stops issuing tool calls. Once done it emits canonical JSONL on stdout and calls `apm state` to transition the ticket. Phi-4's context window is 16 K tokens, so the worker instructions file must be concise enough to leave room for ticket content.

### Acceptance criteria

- [x] `.apm/agents/phi4/manifest.toml` exists and parses without error under `apm validate`
- [x] `manifest.toml` declares `contract_version = 1` and `parser = "canonical"` under `[wrapper]`
- [x] `.apm/agents/phi4/wrapper.py` is executable and exits 0 when Ollama returns a response with no `tool_calls`
- [x] The wrapper reads `APM_SYSTEM_PROMPT_FILE` and `APM_USER_MESSAGE_FILE` from the environment
- [x] The wrapper emits at least one JSONL line with a `"type"` key on stdout before exiting
- [x] When the model returns `tool_calls`, the wrapper executes each tool and appends the result as a `tool` role message before calling the API again
- [x] The `bash` tool executes its `command` argument via a subprocess and returns stdout+stderr
- [x] The `read_file` tool reads and returns the contents of the given `path`
- [x] The `write_file` tool writes `content` to the given `path`, creating parent directories as needed
- [x] The `str_replace` tool replaces the first occurrence of `old_str` with `new_str` in `path`
- [x] After the loop ends, the wrapper calls `apm state $APM_TICKET_ID implemented`
- [x] `.apm/agents/phi4/apm.worker.md` exists and contains both the standard APM worker rules and a `## Tools` section explaining the four function-call tools

### Out of scope

- Wiring `phi4` into `config.toml` as the default or a named worker profile (supervisor action)
- Streaming partial tokens — the wrapper uses non-streaming chat completions
- Authentication / API-key handling for Ollama (it runs unauthenticated locally)
- Error-retry logic beyond a single HTTP failure
- Support for tools beyond the four listed (bash, read_file, write_file, str_replace)
- Changes to any existing file in the repository

### Approach

Three new files under `.apm/agents/phi4/`; no existing files modified.

**manifest.toml** — Create with content:

```toml
[wrapper]
name = "phi4"
contract_version = 1
parser = "canonical"
```

`contract_version = 1` matches the current `CONTRACT_VERSION` constant. `parser = "canonical"` tells APM to scan stdout for JSONL lines (any object with a `"type"` key counts as a canonical event). APM passes on validation.

**wrapper.py** — Python 3 script; shebang `#!/usr/bin/env python3`; chmod +x. Uses only stdlib (`json`, `os`, `pathlib`, `subprocess`, `sys`, `urllib.request`).

Steps inside the script:

1. Read `APM_SYSTEM_PROMPT_FILE`, `APM_USER_MESSAGE_FILE`, `APM_TICKET_ID`, `APM_BIN` from `os.environ`.

2. Declare `TOOLS` — a list of four OpenAI function-calling objects (`type="function"`, `name`, `parameters` JSON Schema):
   `bash(command:str)`, `read_file(path:str)`, `write_file(path:str,content:str)`, `str_replace(path:str,old_str:str,new_str:str)`.

3. Build initial history: `[{"role":"system","content":sys}, {"role":"user","content":msg}]`.

4. Agent loop: POST to `http://localhost:11434/v1/chat/completions` with `model="phi4"`, `tools=TOOLS`, `stream=False` via `urllib.request.urlopen`. Parse JSON response:
   - `finish_reason == "tool_calls"`: append the assistant message, call `run_tool(name, args)` for each tool call, append `{"role":"tool","tool_call_id":...,"content":result}`, loop again.
   - Otherwise: capture `choices[0].message.content` as `final_text` and break.

5. Emit: `print(json.dumps({"type":"result","text":final_text}))` then `sys.stdout.flush()`.

6. Transition: `subprocess.run([apm_bin, "state", ticket_id, "implemented"], check=True)`.

**`run_tool(name, args)` helper:**
- `bash`: `subprocess.run(args["command"], shell=True, capture_output=True, text=True)`; return stdout+stderr capped at 4 000 chars.
- `read_file`: `Path(args["path"]).read_text()`.
- `write_file`: mkdir parents, write text; return `"ok"`.
- `str_replace`: read, `.replace(old_str, new_str, 1)`, write back; return `"ok"`.
- unknown name: return `f"unknown tool: {name}"`.

**apm.worker.md** — Target <= 600 words. Sections: identity sentence, before-coding checklist, permitted apm commands, minimal-change discipline, commit format, finishing step, a "Tools" section with one-line description and a compact JSON call example for each of the four tools, and a blocked/side-note note.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-06T19:06Z | — | new | philippepascal |
| 2026-05-06T20:47Z | new | groomed | philippepascal |
| 2026-05-06T20:48Z | groomed | in_design | philippepascal |
| 2026-05-06T20:52Z | in_design | specd | claude-0506-2048-cdc0 |
| 2026-05-06T22:39Z | specd | ready | philippepascal |
| 2026-05-06T22:40Z | ready | in_progress | philippepascal |
| 2026-05-06T22:46Z | in_progress | implemented | claude-0506-2240-21c0 |
| 2026-05-06T23:39Z | implemented | closed | philippepascal(apm-sync) |