+++
id = "4bee5771"
title = "Enrich apm instructions to emit full APM system knowledge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4bee5771-enrich-apm-instructions-to-emit-full-apm"
created_at = "2026-05-22T23:22:16.080767Z"
updated_at = "2026-05-22T23:51:46.172038Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
+++

## Spec

### Problem

apm instructions currently emits a compact one-liner-per-command summary (apm/src/cmd/instructions.rs). For the prompt redesign it needs to emit full APM system knowledge so transition agents don't need that content duplicated in agents.md. Required additions: (1) state machine — parse workflow.toml and emit all states with their transitions and who can trigger them; (2) ticket format — parse ticket.toml and emit required frontmatter fields and body sections; (3) shell discipline — the Claude Code permission-system constraints (no &&, no &, no $(), use git -C, one command per Bash call); (4) session identity — APM_AGENT_NAME export instructions; (5) keep the existing command reference. The function that generates this text must live in apm-core/src/instructions.rs so both the CLI command and the prompt builder (apm-core/src/start.rs build_system_prompt) can call it. Tests: verify output contains key sections, no ANSI codes, idem­potent.
Scoping requirement: emitting the full state machine to a worker that only touches ready → in_progress → implemented is noise and wastes context. The command needs a --role <name> flag that scopes the output to what is relevant for that role. With no flag the output is generic (full — appropriate for the main agent). The role names must match those defined in the workflow config (worker_profiles and transitions). Scoping affects at minimum: (a) state machine section — emit only the states and transitions the role can be actor of or needs awareness of; (b) command reference — emit only the apm commands the role needs. Shell discipline, session identity, and ticket format are role-independent and always emitted. The library function in apm-core/src/instructions.rs must accept an optional role parameter, so build_system_prompt can pass the resolved role when assembling a transition agent prompt.

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
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-22T23:51Z | groomed | in_design | philippepascal |
