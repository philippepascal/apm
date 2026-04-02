+++
id = "53788218"
title = "apm agents command broken after migration to .apm/ directory"
state = "in_design"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "91210"
branch = "ticket/53788218-apm-agents-command-broken-after-migratio"
created_at = "2026-04-02T05:27:56.648370Z"
updated_at = "2026-04-02T17:08:19.134299Z"
+++

## Spec

### Problem

The `apm agents` command reads the agents instructions file path from `[agents] instructions` in `.apm/config.toml`. During the migration that moved the agents file from `apm.agents.md` (repo root) to `.apm/agents.md`, the path stored in `.apm/config.toml` was not updated. As a result, running `apm agents` fails with a "No such file or directory" error because the old filename `apm.agents.md` no longer exists.\n\nThe agents instructions file is the single source of truth for agent behaviour in a project. When `apm agents` is broken, users cannot inspect or validate what instructions agents are operating under, and any tooling that pipes `apm agents` output into system prompts also fails.

### Acceptance criteria

- [ ] `apm agents` exits with code 0 and prints the contents of `.apm/agents.md`\n- [ ] `apm agents` does not print an error about a missing file\n- [ ] No other `apm` commands are broken by the change

### Out of scope

- Changing the `apm agents` command logic or output format\n- Migrating any other files or config keys that may still reference old paths\n- Adding validation that the instructions file exists at config-load time

### Approach

Single-file config change — no code changes required.\n\n1. Open `.apm/config.toml` (the project's own APM config, at the repo root)\n2. On the line `instructions = "apm.agents.md"` under `[agents]`, update it to `instructions = ".apm/agents.md"`\n3. Verify manually: `cargo run -p apm -- agents` should print the contents of `.apm/agents.md`\n4. Commit the change to the ticket branch:\n   `git -C <worktree> add .apm/config.toml`\n   `git -C <worktree> commit -m "Fix agents instructions path after .apm/ migration"`\n\nNo test changes needed — the existing test suite covers config loading; the fix is a data change, not a logic change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T05:27Z | — | new | apm |
| 2026-04-02T16:58Z | new | groomed | apm |
| 2026-04-02T17:05Z | groomed | in_design | philippepascal |