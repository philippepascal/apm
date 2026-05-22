+++
id = "4bee5771"
title = "Enrich apm instructions to emit full APM system knowledge"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4bee5771-enrich-apm-instructions-to-emit-full-apm"
created_at = "2026-05-22T23:22:16.080767Z"
updated_at = "2026-05-22T23:22:16.080767Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
+++

## Spec

### Problem

apm instructions currently emits a compact one-liner-per-command summary (apm/src/cmd/instructions.rs). For the prompt redesign it needs to emit full APM system knowledge so transition agents don't need that content duplicated in agents.md. Required additions: (1) state machine — parse workflow.toml and emit all states with their transitions and who can trigger them; (2) ticket format — parse ticket.toml and emit required frontmatter fields and body sections; (3) shell discipline — the Claude Code permission-system constraints (no &&, no &, no $(), use git -C, one command per Bash call); (4) session identity — APM_AGENT_NAME export instructions; (5) keep the existing command reference. The function that generates this text must live in apm-core/src/instructions.rs so both the CLI command and the prompt builder (apm-core/src/start.rs build_system_prompt) can call it. Tests: verify output contains key sections, no ANSI codes, idem­potent.

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
| 2026-05-22T23:22Z | — | new | philippepascal |
