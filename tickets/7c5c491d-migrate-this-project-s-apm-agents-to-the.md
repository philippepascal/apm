+++
id = "7c5c491d"
title = "Migrate this project's .apm/agents/ to the new structure"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5c491d-migrate-this-project-s-apm-agents-to-the"
created_at = "2026-05-22T23:23:29.954873Z"
updated_at = "2026-05-23T00:30:48.579585Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["34ad9126", "78eeb755", "02bbcc2f", "1fce91bd", "7ef960f2"]
+++

## Spec

### Problem

The APM project's own `.apm/agents/` directory was created from the original monolithic `agents.md` built-in default. With the prompt management redesign (epic ab6e5db7), that monolith is being split into three composed layers: dynamic APM system knowledge from `apm instructions` (T1/4bee5771), project context from `apm.project.md`, and role-specific instructions from role files. The sibling tickets (T2–T8) update the built-ins and the `apm init` scaffold, but none of them update this project's own `.apm/agents/` directory.

Until this ticket is implemented, the project's agents receive stale instructions — specifically: `agents.md` still referenced as the single instructions file, role files still contain shell-discipline and session-identity content that will be covered by `apm instructions`, `CLAUDE.md` imports only `agents.md`, and `.apm/config.toml` still uses `instructions =` rather than the renamed `project =` key that T3 introduces.

The desired end state: `agents.md` deleted; two new files (`apm.project.md`, `apm.main-agent.md`) created with project-specific content; `apm.spec-writer.md` and `apm.worker.md` updated to match the rewritten built-ins; `claude/apm.spec-writer.md` and `claude/apm.worker.md` deleted (both are stale overrides that should fall through to the updated defaults); `CLAUDE.md` updated to import the two new files; and `.apm/config.toml` `[agents]` `instructions` key renamed to `project`.

### Acceptance criteria

- [ ] `.apm/agents/default/agents.md` does not exist in the repo
- [ ] `.apm/agents/default/apm.project.md` exists and contains APM-specific project context (crate structure, module responsibilities)
- [ ] `.apm/agents/default/apm.main-agent.md` exists and matches the built-in default created by edb0cf35
- [ ] `.apm/agents/default/apm.spec-writer.md` matches the rewritten built-in from 34ad9126 (no runtime notice, no permitted-commands list, no shell-discipline block in § How to save spec sections, amendment step 6 references auto-commit not a manual git block)
- [ ] `.apm/agents/default/apm.worker.md` matches the rewritten built-in from 78eeb755 (no `agents.md` back-reference, no `## Shell discipline` section, has `## Ticket file discipline`)
- [ ] `.apm/agents/claude/apm.worker.md` does not exist in the repo
- [ ] `.apm/agents/claude/apm.spec-writer.md` does not exist in the repo
- [ ] `CLAUDE.md` contains `@.apm/agents/default/apm.project.md` and `@.apm/agents/default/apm.main-agent.md`
- [ ] `CLAUDE.md` does not contain `@.apm/agents/default/agents.md`
- [ ] `.apm/config.toml` `[agents]` section has `project = ".apm/agents/default/apm.project.md"` and does not contain an `instructions =` key

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
| 2026-05-22T23:51Z | new | groomed | philippepascal |
| 2026-05-23T00:30Z | groomed | in_design | philippepascal |