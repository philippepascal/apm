+++
id = "d8e2fa0e"
title = "Redesign build_system_prompt to compose three layers"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d8e2fa0e-redesign-build-system-prompt-to-compose-"
created_at = "2026-05-22T23:23:06.850140Z"
updated_at = "2026-05-22T23:50:55.632303Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771", "edb0cf35"]
+++

## Spec

### Problem

build_system_prompt (apm-core/src/start.rs:901) currently works as: prepend agents.instructions file → pick one cascade winner (per-agent file | transition | profile | workers | built-in). The new model composes three layers: (1) apm instructions output (from T1's library function in apm-core/src/instructions.rs), (2) apm.project.md content (from a configurable path, default .apm/agents/default/apm.project.md), (3) role file (the existing cascade, unchanged). The config.toml [agents] section gets a new key: project = ".apm/agents/default/apm.project.md" replacing instructions = ".apm/agents/default/agents.md". The instructions key is deprecated — if present, emit a deprecation warning and ignore it (or treat as project for one release). update build_system_prompt, build_system_prompt_body, and explain_system_prompt accordingly. The explain output (--explain flag) must show all three layer sources: apm instructions (dynamic), project file path, role file path.

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
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
