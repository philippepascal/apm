+++
id = "7c5c491d"
title = "Migrate this project's .apm/agents/ to the new structure"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5c491d-migrate-this-project-s-apm-agents-to-the"
created_at = "2026-05-22T23:23:29.954873Z"
updated_at = "2026-05-22T23:24:29.857517Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["34ad9126", "78eeb755", "02bbcc2f", "1fce91bd", "7ef960f2"]
+++

## Spec

### Problem

This project's own .apm/agents/ directory must be migrated to the new structure. Files to change: (1) delete .apm/agents/default/agents.md (currently 339 lines — content redistributed to apm.project.md, apm.main-agent.md, apm instructions); (2) create .apm/agents/default/apm.project.md with APM-specific project context: what APM is, the stack (Rust workspace: apm-core, apm, apm-server), module responsibilities, repo structure, key technical decisions; (3) create .apm/agents/default/apm.main-agent.md matching the updated built-in but with any project-specific additions; (4) update .apm/agents/default/apm.spec-writer.md to match the rewritten built-in; (5) update .apm/agents/default/apm.worker.md to match the rewritten built-in; (6) delete .apm/agents/claude/apm.worker.md (identical to default, deleted in T6); (7) update CLAUDE.md: replace @.apm/agents/default/agents.md with @.apm/agents/default/apm.project.md and @.apm/agents/default/apm.main-agent.md; (8) update .apm/config.toml [agents] section: rename instructions key to project (from T3 config change). Run apm prompt --agent claude --role worker after to verify the assembled prompt has the right content.

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