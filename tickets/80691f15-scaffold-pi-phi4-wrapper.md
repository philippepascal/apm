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
| 2026-05-06T21:28Z | groomed | in_design | philippepascal |