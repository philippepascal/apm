+++
id = "53788218"
title = "apm agents command broken after migration to .apm/ directory"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "93051"
branch = "ticket/53788218-apm-agents-command-broken-after-migratio"
created_at = "2026-04-02T05:27:56.648370Z"
updated_at = "2026-04-02T19:06:40.007733Z"
+++

## Spec

### Problem

The `apm agents` command reads the agents instructions file path from `[agents] instructions` in `.apm/config.toml`. During the migration that moved the agents file from `apm.agents.md` (repo root) to `.apm/agents.md`, the path stored in `.apm/config.toml` was not updated. As a result, running `apm agents` fails with a "No such file or directory" error because the old filename `apm.agents.md` no longer exists.\n\nThe agents instructions file is the single source of truth for agent behaviour in a project. When `apm agents` is broken, users cannot inspect or validate what instructions agents are operating under, and any tooling that pipes `apm agents` output into system prompts also fails.

### Acceptance criteria

- [x] `apm agents` exits with code 0 and prints the contents of `.apm/agents.md`
- [x] `apm agents` does not print an error about a missing file
- [x] No other `apm` commands are broken by the change

### Out of scope

- Changing the `apm agents` command logic or output format\n- Migrating any other files or config keys that may still reference old paths\n- Adding validation that the instructions file exists at config-load time

### Approach

Update `instructions` under `[agents]` in `.apm/config.toml` from `apm.agents.md` to `.apm/agents.md`.

Add an integration test in `apm/tests/integration.rs` that:
- Creates a temp git repo with an apm.toml that sets `[agents] instructions = "agents-instructions.md"`
- Writes a file `agents-instructions.md` with known content to the repo root
- Calls `apm::cmd::agents::run(&root)` (or captures stdout and calls the binary)
- Asserts the call returns `Ok(())` and the file contents are printed to stdout

The test should use the existing `setup()` helper pattern from the integration test file and capture stdout via a suitable mechanism (e.g. `std::io::Write` redirect or capturing `print!` output). Look at how similar tests call `apm::cmd` functions directly.

### Open questions


### Amendment requests

- [x] Simplify the Approach — remove the exact git commands and manual verification steps. The worker knows how to commit; just say "update `instructions` under `[agents]` in `.apm/config.toml` to point to `.apm/agents.md`". The approach should state what to change, not how to run git.
- [x] Add a test — `apm agents` is a CLI command; add an integration test (or check if one exists) that verifies `apm agents` exits 0 and outputs the file contents when the path is correctly configured. "No test changes needed" is not acceptable for a fix that has a clear observable behaviour.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:27Z | — | new | apm |
| 2026-04-02T16:58Z | new | groomed | apm |
| 2026-04-02T17:05Z | groomed | in_design | philippepascal |
| 2026-04-02T17:08Z | in_design | specd | claude-0402-1710-b7f2 |
| 2026-04-02T17:27Z | specd | ammend | apm |
| 2026-04-02T17:27Z | ammend | in_design | philippepascal |
| 2026-04-02T17:29Z | in_design | specd | claude-0402-1800-c9d1 |
| 2026-04-02T17:39Z | specd | ready | apm |
| 2026-04-02T17:39Z | ready | in_progress | philippepascal |
| 2026-04-02T17:45Z | in_progress | implemented | claude-0402-1745-f2e1 |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |