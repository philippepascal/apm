+++
id = "95b9279d"
title = "apm prompt --explain: show cascade provenance instead of prompt text"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/95b9279d-apm-prompt-explain-show-cascade-provenan"
created_at = "2026-05-22T10:22:16.387302Z"
updated_at = "2026-05-22T10:23:20.934810Z"
+++

## Spec

### Problem

build_system_prompt() applies a 5-level cascade to resolve the system prompt: (0) .apm/agents/<agent>/apm.<role>.md, (1) transition.instructions, (2) profile.instructions, (3) workers.instructions, (4) built-in default — plus an agents.instructions prefix prepended on top. When a worker behaves unexpectedly, there is no way to tell which level won or which file was actually read without grepping config files manually.

apm prompt <id> --explain should print the provenance of the assembled prompt instead of the prompt itself: which file was used as the agents.instructions prefix (if any), which level of the cascade won and what file or config path it came from, and which levels were checked and skipped. Example output:

  prefix:         .apm/agents/default/agents.md  (agents.instructions)
  system prompt:  .apm/agents/claude/apm.worker.md  (level 0 — per-agent file)
  skipped:        level 1 (transition.instructions — none set)
                  level 2 (profile.instructions — none set)
                  level 3 (workers.instructions — none set)

This is a debugging tool for supervisors to verify prompt resolution without reading a full prompt dump.

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
| 2026-05-22T10:22Z | — | new | philippepascal |
| 2026-05-22T10:23Z | new | groomed | philippepascal |
