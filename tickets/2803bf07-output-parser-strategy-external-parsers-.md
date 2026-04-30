+++
id = "2803bf07"
title = "Output parser strategy: external parsers via manifest.toml"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2803bf07-output-parser-strategy-external-parsers-"
created_at = "2026-04-30T20:05:40.844536Z"
updated_at = "2026-04-30T21:02:46.987871Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["2c32a282", "2e772eab"]
+++

## Spec

### Problem

Support agents whose output is too far from APM's canonical JSONL stream-json to translate inline. The wrapper declares an external parser binary in `manifest.toml`; APM pipes the wrapper's stdout through it before capturing.

**Reference spec:** `docs/agent-wrappers.md` — section 'Output parser strategy', 'Custom wrappers / manifest.toml'.

**Scope:**
- `manifest.toml` already parses `parser` and `parser_command` (added in ticket 2c32a282). This ticket wires them into spawn.
- Three parser modes:
  - `parser = "canonical"` (default) — wrapper produces JSONL stream-json directly. No transformation. Today's behaviour.
  - `parser = "raw"` — wrapper output is captured as-is to log; no canonical-event parsing. Useful for agents whose output is unstructured or only meant for human reading. Worker-state events still drive off the wrapper's exit code and any `apm state` calls it makes.
  - `parser = "external"` — wrapper output is piped through the binary at `parser_command` (must be in PATH or absolute path); the parser's stdout becomes APM's captured stream. Parser must produce canonical JSONL.
- Spawn glue: when `parser = "external"`, spawn the wrapper and the parser as a pipe (wrapper.stdout → parser.stdin). Capture parser.stdout (canonical events) and parser.stderr (parser's diagnostics) to `.apm-worker.log`. The wrapper's stderr also goes to the log directly.
- Validate `parser_command` exists when `parser = "external"`. Fail at spawn with a clear error if not.
- Built-in wrappers always default to canonical (no manifest needed).

**Out of scope:**
- Shipping any external parser binaries (e.g. `apm-output-parser-aider`). Those are separate cargo crates.
- A formal canonical event vocabulary doc — already noted as an open question in the spec doc.
- Multiplexing two parsers on the same wrapper. Pick one strategy per wrapper.

**Tests:**
- `canonical` mode: existing wrapper tests unchanged.
- `raw` mode: a wrapper that emits "hello world" → log contains "hello world" verbatim, no JSONL parse warnings.
- `external` mode: a wrapper that emits non-JSONL text + a parser script that wraps each line in a JSONL event → log contains the parsed JSONL, original text only in the parser's stderr.
- Missing parser_command → spawn fails with a clear error citing the manifest path.

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
| 2026-04-30T20:05Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
