+++
id = "edb0cf35"
title = "Create apm.project.md template and apm.main-agent.md built-in defaults"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/edb0cf35-create-apm-project-md-template-and-apm-m"
created_at = "2026-05-22T23:22:36.259605Z"
updated_at = "2026-05-23T00:09:26.920145Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

Two new built-in content files are needed in apm-core/src/default/agents/default/. First: apm.project.md — a placeholder template the user fills in with project-specific context: what we are building, tech stack, technical decisions, module responsibilities, repo structure. It should have clear section headers and placeholder text so fresh apm init projects know what to put there. Second: apm.main-agent.md — the supervisor companion role file (no role detection, that is handled by prompt assembly). Content: what the main agent does (helps supervisor create tickets, review specs, manage epics), what it does NOT do (spawn workers, push code unsolicited, transition states without authorization), supervisor-only transitions list, override clause, startup sequence (sync, next, list — and run apm instructions at session start). Both files must be include_str! compiled into apm-core (same as existing role files). The startup sequence in apm.main-agent.md should reference apm instructions as the source of APM system knowledge, not duplicate it.

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
| 2026-05-23T00:09Z | groomed | in_design | philippepascal |
