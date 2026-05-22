+++
id = "02bbcc2f"
title = "Remove redundant claude/apm.worker.md built-in default"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/02bbcc2f-remove-redundant-claude-apm-worker-md-bu"
created_at = "2026-05-22T23:22:45.436649Z"
updated_at = "2026-05-22T23:50:50.931754Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["78eeb755"]
+++

## Spec

### Problem

apm-core/src/default/agents/claude/apm.worker.md is byte-for-byte identical to apm-core/src/default/agents/default/apm.worker.md — it adds zero value and creates a maintenance burden. After T5 rewrites the default worker.md, this override should be deleted. Changes: (1) delete apm-core/src/default/agents/claude/apm.worker.md; (2) remove const CLAUDE_WORKER_DEFAULT in apm-core/src/start.rs:7; (3) update resolve_builtin_instructions() to remove the ("claude", "worker") => Some(CLAUDE_WORKER_DEFAULT) arm — it will fall through to the default lookup or the per-agent file in the project's .apm/agents/claude/ directory. Integration tests that assert on the claude/worker prompt content will need to be updated.

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
