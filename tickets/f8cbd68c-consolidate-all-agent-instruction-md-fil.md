+++
id = "f8cbd68c"
title = "Consolidate all agent instruction .md files under agents/*/"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f8cbd68c-consolidate-all-agent-instruction-md-fil"
created_at = "2026-05-04T02:41:12.168717Z"
updated_at = "2026-05-04T03:05:20.537191Z"
+++

## Spec

### Problem

All agent instruction .md files should live under agents/*/. Currently they are scattered:

Project .apm/:
  .apm/agents.md                         → .apm/agents/default/agents.md
  .apm/apm.spec-writer.md                → .apm/agents/default/apm.spec-writer.md
  .apm/apm.worker.md                     → .apm/agents/default/apm.worker.md
  .apm/style.md                          → .apm/agents/default/style.md
  .apm/agents/claude/apm.spec-writer.md  (already correct, stays)
  .apm/agents/claude/apm.worker.md       (already correct, stays)

Embedded defaults (apm-core/src/default/):
  apm.agents.md        → agents/default/agents.md
  apm.spec-writer.md   → agents/default/apm.spec-writer.md
  apm.worker.md        → agents/default/apm.worker.md
  agents/claude/…      (already correct, stays)
  agents/debug/…       (already correct, stays)
  agents/mock-*/…      (already correct, stays)

Explicit path updates required (no fallback resolution):
  apm-core/src/init.rs:
    - write_default paths: .apm/apm.spec-writer.md → .apm/agents/default/apm.spec-writer.md, etc.
    - include_str!() paths: default/apm.*.md → default/agents/default/*.md
    - Config template string (line 306): instructions = ".apm/agents.md" → ".apm/agents/default/agents.md"
    - Worker profile strings (lines 315, 320): similar updates
    - Migration: add a migration step to rewrite old paths in CLAUDE.md and config files for existing projects
  apm-core/src/default/workflow.toml: 5 instructions = lines pointing to .apm/apm.spec-writer.md and .apm/apm.worker.md → new paths
  .apm/config.toml (project): instructions fields for agents, spec_agent, impl_agent
  .apm/workflow.toml (project): instructions fields
  CLAUDE.md: @.apm/agents.md → @.apm/agents/default/agents.md, @.apm/style.md → @.apm/agents/default/style.md, and prose references
  apm-core/tests/worker_md_sync.rs: update comparison paths
  apm-core/tests/spec_writer_md_sync.rs: no change needed (already compares agents/claude/)

Conflict: ticket 121a05a8 (specd) writes per-agent files from init.rs and adds sync tests. Its Step 4 (init.rs) and test paths will conflict with this refactor. This ticket should be implemented first or 121a05a8 rebased on top of it.

### Acceptance criteria

- [ ] `apm-core/src/default/agents/default/` contains `agents.md`, `apm.spec-writer.md`, and `apm.worker.md`; the old flat files `apm-core/src/default/apm.agents.md`, `apm-core/src/default/apm.spec-writer.md`, and `apm-core/src/default/apm.worker.md` no longer exist
- [ ] `apm init` on a fresh project creates `.apm/agents/default/agents.md` and does not create `.apm/agents.md`
- [ ] `apm init` on a fresh project creates `.apm/agents/default/apm.spec-writer.md` and does not create `.apm/apm.spec-writer.md`
- [ ] `apm init` on a fresh project creates `.apm/agents/default/apm.worker.md` and does not create `.apm/apm.worker.md`
- [ ] `apm init` on a fresh project writes `CLAUDE.md` containing `@.apm/agents/default/agents.md`
- [ ] `apm init` on a project whose `.apm/` still has old flat files (`agents.md`, `apm.spec-writer.md`, `apm.worker.md`, `style.md`) moves each one to `.apm/agents/default/` and leaves no file at the old path
- [ ] After the migration path above, CLAUDE.md references, `config.toml` `instructions` fields, and `workflow.toml` `instructions` fields are all rewritten from old paths to new paths
- [ ] `apm-core/src/default/workflow.toml` contains no references to `.apm/apm.spec-writer.md` or `.apm/apm.worker.md`
- [ ] This repo's `.apm/agents/default/` contains `agents.md`, `apm.spec-writer.md`, `apm.worker.md`, and `style.md`
- [ ] This repo's `CLAUDE.md` imports `@.apm/agents/default/agents.md` and `@.apm/agents/default/style.md`
- [ ] `cargo test --workspace` passes

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
| 2026-05-04T02:41Z | — | new | philippepascal |
| 2026-05-04T02:56Z | new | groomed | philippepascal |
| 2026-05-04T03:05Z | groomed | in_design | philippepascal |