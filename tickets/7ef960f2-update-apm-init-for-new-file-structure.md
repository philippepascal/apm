+++
id = "7ef960f2"
title = "Update apm init for new file structure"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ef960f2-update-apm-init-for-new-file-structure"
created_at = "2026-05-22T23:23:20.147068Z"
updated_at = "2026-05-23T00:25:32.538611Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["edb0cf35", "d8e2fa0e", "02bbcc2f", "1fce91bd"]
+++

## Spec

### Problem

`apm init` (`apm-core/src/init.rs setup()`) was designed around the old monolithic `agents.md` file structure. With the prompt management redesign (epic ab6e5db7), agent instruction files are split into three composed layers: dynamic APM knowledge from `apm instructions`, project context from `apm.project.md`, and role-specific instructions from role files. Four sibling tickets restructure those files — T2 (edb0cf35) creates the new built-in defaults, T3 (d8e2fa0e) renames the `[agents] instructions` config key to `project`, T6 (02bbcc2f) removes the redundant `claude/apm.worker.md` built-in, and T7 (1fce91bd) deletes the `agents.md` built-in.

This ticket wires those changes into `init.rs`: `setup()` must write the two new files instead of the old one, stop writing the redundant `claude/apm.worker.md`, inject the correct `@` imports into CLAUDE.md, and emit `project = ...` in the generated config. `migrate_flat_agent_files` must additionally handle existing projects that still reference `agents.md` — both in CLAUDE.md `@` imports and in config.toml `instructions = ...` keys — upgrading them to the new paths and key name in a single migration pass.

### Acceptance criteria

- [ ] `apm init` creates `.apm/agents/default/apm.project.md` using the built-in default template from T2 (edb0cf35)
- [ ] `apm init` creates `.apm/agents/default/apm.main-agent.md` using the built-in default from T2 (edb0cf35)
- [ ] `apm init` does not create `.apm/agents/default/agents.md`
- [ ] `apm init` does not create `.apm/agents/claude/apm.worker.md`
- [ ] `apm init` still creates `.apm/agents/claude/apm.spec-writer.md` (content differs from default, kept per-agent)
- [ ] A freshly initialized CLAUDE.md contains `@.apm/agents/default/apm.project.md` and `@.apm/agents/default/apm.main-agent.md` and does not contain `@.apm/agents/default/agents.md`
- [ ] The generated `config.toml` contains `project = ".apm/agents/default/apm.project.md"` in `[agents]` and does not contain `instructions = ".apm/agents/default/agents.md"`
- [ ] Running `apm init` on a project whose CLAUDE.md contains `@.apm/agents/default/agents.md` replaces that line with both new `@` imports
- [ ] Running `apm init` on a project whose `config.toml` has `instructions = ".apm/agents/default/agents.md"` rewrites it to `project = ".apm/agents/default/apm.project.md"`
- [ ] `cargo test --workspace` passes with all init tests updated to reflect the new structure

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
| 2026-05-23T00:25Z | groomed | in_design | philippepascal |