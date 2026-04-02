+++
id = "53788218"
title = "apm agents command broken after migration to .apm/ directory"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "91210"
branch = "ticket/53788218-apm-agents-command-broken-after-migratio"
created_at = "2026-04-02T05:27:56.648370Z"
updated_at = "2026-04-02T17:05:42.504353Z"
+++

## Spec

### Problem

The `apm agents` command reads the agents instructions file path from `[agents] instructions` in `.apm/config.toml`. During the migration that moved the agents file from `apm.agents.md` (repo root) to `.apm/agents.md`, the path stored in `.apm/config.toml` was not updated. As a result, running `apm agents` fails with a "No such file or directory" error because the old filename `apm.agents.md` no longer exists.\n\nThe agents instructions file is the single source of truth for agent behaviour in a project. When `apm agents` is broken, users cannot inspect or validate what instructions agents are operating under, and any tooling that pipes `apm agents` output into system prompts also fails.

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
| 2026-04-02T05:27Z | — | new | apm |
| 2026-04-02T16:58Z | new | groomed | apm |
| 2026-04-02T17:05Z | groomed | in_design | philippepascal |