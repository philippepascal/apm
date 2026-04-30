+++
id = "71d80e40"
title = "apm agents subcommand: new, list, test, eject"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/71d80e40-apm-agents-subcommand-new-list-test-ejec"
created_at = "2026-04-30T20:04:57.796154Z"
updated_at = "2026-04-30T22:02:56.837073Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

Add the `apm agents` subcommand family for discovering, scaffolding, smoke-testing, and ejecting wrappers. Discoverability and authoring are the load-bearing UX for the wrapper feature.

**Reference spec:** `docs/agent-wrappers.md` — sections 'Skeleton command', 'Other wrapper-related commands'.

**Scope:** four subcommands under `apm agents`.

**`apm agents list`** — discover available wrappers.
- Lists built-in wrappers (from the registry in d3b93b95 + mocks from 25c92daa) and project-defined wrappers (from `.apm/agents/<name>/wrapper.*` per 2c32a282).
- For each: name, kind (built-in or project), and current configured-as marker (which profile/global uses it).
- One column for parser strategy if the wrapper declares one in manifest.toml.

**`apm agents new <name>`** — scaffold a custom wrapper.
- Creates `.apm/agents/<name>/` if it doesn't exist; refuses if it does (suggest `--force` for overwrite).
- Writes:
  - `wrapper.sh` — runnable template that prints all `APM_*` env vars to stderr, emits a minimal valid JSONL event on stdout, exits 0. Documents the contract inline as comments. Sets the execute bit (`chmod +x`).
  - `apm.worker.md` — copy of the project's current `.apm/apm.worker.md` (or the claude built-in's default if no project file).
  - `apm.spec-writer.md` — same.
  - `manifest.toml` — defaults written explicitly: `contract_version = 1`, `parser = "canonical"`.
- Prints next-step guidance: edit `wrapper.sh`, run `apm agents test <name>` to validate.

**`apm agents test <name>`** — smoke-test a wrapper.
- Spawns the wrapper against a synthetic ticket in a temp worktree (no real ticket touched).
- Captures the wrapper's output and exit code.
- Reports: exit code, count of canonical JSONL events, any non-canonical lines on stdout, count of stderr lines, wall time.
- Pass criteria: exit 0, at least one canonical JSONL event, no parse errors.
- Useful before assigning a new wrapper to a real worker queue.

**`apm agents eject <name>`** — extract a built-in to a script.
- Writes the built-in wrapper's source equivalent to `.apm/agents/<name>/wrapper.sh` (a bash script that reproduces the built-in's behaviour). The Rust built-in stays registered; the project script shadows it per the resolution rules in 2c32a282.
- Useful when a user wants to customize a built-in (e.g. add custom env vars, change the model invocation).
- Refuses if `.apm/agents/<name>/` already exists.

**Out of scope:**
- Wrapper-contract version checking inside `apm agents test` — defer to the versioning ticket.
- Distributing wrappers across projects (`apm agents install`) — out of scope.
- An `apm agents remove` command — users can `rm -r` the directory.

**Tests:**
- `list`: built-ins appear; a fixture project script appears with kind=project.
- `new`: directory and files created; `wrapper.sh` is executable; second invocation refuses.
- `test`: passing wrapper reports success; failing wrapper (non-zero exit) reports the failure with the captured stderr.
- `eject`: claude eject writes a script that, when run as the configured agent, produces the same canonical events as the built-in.

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
| 2026-04-30T20:04Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T22:02Z | groomed | in_design | philippepascal |
