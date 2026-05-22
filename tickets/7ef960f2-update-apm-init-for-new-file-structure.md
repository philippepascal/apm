+++
id = "7ef960f2"
title = "Update apm init for new file structure"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ef960f2-update-apm-init-for-new-file-structure"
created_at = "2026-05-22T23:23:20.147068Z"
updated_at = "2026-05-22T23:24:24.678072Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["edb0cf35", "d8e2fa0e", "02bbcc2f", "1fce91bd"]
+++

## Spec

### Problem

apm init (apm-core/src/init.rs setup()) must be updated to write the new file structure. Changes: (1) write apm.project.md template (from T2's built-in) to .apm/agents/default/apm.project.md; (2) write apm.main-agent.md (from T2's built-in) to .apm/agents/default/apm.main-agent.md; (3) stop writing agents.md (deleted in T7); (4) stop writing .apm/agents/claude/apm.worker.md (deleted in T6) — keep claude/apm.spec-writer.md if it has meaningful content vs default; (5) update ensure_claude_md (init.rs:363) to add @.apm/agents/default/apm.project.md and @.apm/agents/default/apm.main-agent.md to CLAUDE.md instead of @.apm/agents/default/agents.md; (6) update migrate_flat_agent_files to recognize and migrate old agents.md paths in CLAUDE.md config references; (7) update default_config to emit project = ".apm/agents/default/apm.project.md" in [agents] instead of instructions = ".apm/agents/default/agents.md" (from T3 config change); (8) update ALL init tests in apm-core/src/init.rs: setup_creates_expected_files, migrate tests, CLAUDE.md content assertions. The ensure_claude_md function should add both @-includes at the top of CLAUDE.md.

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