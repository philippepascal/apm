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
updated_at = "2026-05-01T00:21:33.818831Z"
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

- [ ] `apm agents list` prints a row for the `claude` built-in with kind `built-in`
- [ ] `apm agents list` prints a row for each executable `wrapper.*` found under `.apm/agents/<name>/` with kind `project`
- [ ] `apm agents list` marks the agent matching the configured `workers.command` (legacy field) with a `(configured)` indicator
- [ ] `apm agents list` shows a `parser` column value read from `manifest.toml`; defaults to `canonical` when the manifest is absent or the field is unset
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/wrapper.sh` with the execute bit set (mode `0o755` on Unix)
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/apm.worker.md` with content copied from `.apm/apm.worker.md` or the built-in default when the project file is absent
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/apm.spec-writer.md` with content copied from `.apm/apm.spec-writer.md` or the built-in default when the project file is absent
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/manifest.toml` containing `contract_version = 1` and `parser = "canonical"`
- [ ] `apm agents new <name>` exits non-zero with a message that mentions `--force` when `.apm/agents/<name>/` already exists
- [ ] `apm agents new <name> --force` succeeds when the directory already exists and overwrites the scaffolded files
- [ ] `apm agents new <name>` prints next-step guidance directing the user to edit `wrapper.sh` and run `apm agents test <name>`
- [ ] `apm agents test <name>` exits 0 and prints a pass summary when the wrapper exits 0 and emits at least one canonical JSONL line (a JSON object containing a `"type"` key)
- [ ] `apm agents test <name>` exits non-zero and prints a fail summary when the wrapper exits non-zero
- [ ] `apm agents test <name>` reports exit code, canonical JSONL event count, non-canonical log line count, stderr line count, and wall-clock milliseconds in its output
- [ ] `apm agents test <name>` exits non-zero with a clear error message when `<name>` is not a known wrapper (built-in or project)
- [ ] `apm agents eject claude` creates `.apm/agents/claude/wrapper.sh` containing a bash script that invokes the `claude` CLI with `--print --output-format stream-json --verbose`
- [ ] `apm agents eject <name>` creates `.apm/agents/<name>/manifest.toml` with `contract_version = 1` and `parser = "canonical"`
- [ ] `apm agents eject <name>` sets the execute bit on the ejected `wrapper.sh`
- [ ] `apm agents eject <name>` exits non-zero when `.apm/agents/<name>/` already exists
- [ ] `apm agents eject <name>` exits non-zero with a message when `<name>` is not a known built-in

### Out of scope

- Per-agent instruction file resolution (`.apm/agents/<name>/apm.worker.md` etc.) — ticket 7f5f73d5; `apm agents new` writes these files as scaffold aids but their resolution order is not wired here
- `apm agents install` and `apm agents remove` subcommands
- Wrapper-contract version checking inside `apm agents test` — ticket 2e772eab
- Mock built-in wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) appearing in `apm agents list` — ticket 25c92daa must land first; once it does they appear automatically via the built-in registry without changes here
- The previous `apm agents` (no subcommand) behaviour of printing the instructions file — this ticket replaces it entirely
- Running `apm agents test` against the real `claude` CLI in automated tests — fixtures use small shell scripts
- Config-driven active-profile column once ticket 6cac8518 lands — pre-6cac8518 the marker uses `workers.command`; `list_wrappers` includes a TODO comment for the post-6cac8518 switch to `workers.agent` and per-profile iteration
- Windows execute-bit semantics (same limitation as ticket 2c32a282; any `wrapper.*` file is treated as executable on non-Unix platforms)

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
| 2026-05-01T00:08Z | in_design | ammend | philippepascal |
| 2026-05-01T00:21Z | ammend | in_design | philippepascal |