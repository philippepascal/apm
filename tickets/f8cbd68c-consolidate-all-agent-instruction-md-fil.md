+++
id = "f8cbd68c"
title = "Consolidate all agent instruction .md files under agents/*/"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f8cbd68c-consolidate-all-agent-instruction-md-fil"
created_at = "2026-05-04T02:41:12.168717Z"
updated_at = "2026-05-04T03:19:50.666271Z"
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

- [x] `apm-core/src/default/agents/default/` contains `agents.md`, `apm.spec-writer.md`, and `apm.worker.md`; the old flat files `apm-core/src/default/apm.agents.md`, `apm-core/src/default/apm.spec-writer.md`, and `apm-core/src/default/apm.worker.md` no longer exist
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

- Changing the content of any agent instruction file (agents.md, apm.spec-writer.md, apm.worker.md, style.md)
- Per-agent overrides already at correct paths (`agents/claude/`, `agents/debug/`, `agents/mock-*/`)
- Fallback path resolution at runtime — paths are explicit, no fallback logic is added
- Ticket 121a05a8, which adds per-agent file writes from init.rs and new sync tests; that ticket must be rebased on top of this one

### Approach

All changes are mechanical path renames — no content edits to any agent instruction file.

#### Step 1 — Move embedded defaults

In `apm-core/src/default/` use `git mv`:
- `apm.agents.md` → `agents/default/agents.md`
- `apm.spec-writer.md` → `agents/default/apm.spec-writer.md`
- `apm.worker.md` → `agents/default/apm.worker.md`

Create the `agents/default/` directory first if `git mv` requires it.

#### Step 2 — Update init.rs

`apm-core/src/init.rs`:

- `default_agents_md()`: change `include_str!("default/apm.agents.md")` → `include_str!("default/agents/default/agents.md")`
- Two `write_default` calls for spec-writer and worker: change `include_str!` paths to `"default/agents/default/apm.spec-writer.md"` and `"default/agents/default/apm.worker.md"`
- In `setup()`, create `agents/default/` dir alongside the existing `agents/claude/` dir creation (line ~135)
- Update the three `apm_dir.join(...)` calls:
  - `"agents.md"` → `"agents/default/agents.md"`
  - `"apm.spec-writer.md"` → `"agents/default/apm.spec-writer.md"`
  - `"apm.worker.md"` → `"agents/default/apm.worker.md"`
- Update `ensure_claude_md` call argument: `".apm/agents.md"` → `".apm/agents/default/agents.md"`
- In `default_config()` template string (around line 321–336), update three `instructions =` values:
  - `[agents] instructions = ".apm/agents.md"` → `".apm/agents/default/agents.md"`
  - `[worker_profiles.spec_agent] instructions = ".apm/apm.spec-writer.md"` → `".apm/agents/default/apm.spec-writer.md"`
  - `[worker_profiles.impl_agent] instructions = ".apm/apm.worker.md"` → `".apm/agents/default/apm.worker.md"`

#### Step 3 — Add migration sub-routine in setup()

Before the `write_default` calls in `setup()`, insert a block that handles existing projects:

1. Ensure `agents/default/` dir exists
2. For each old flat file (`.apm/agents.md`, `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`, `.apm/style.md`): if the old path exists and the new path does not, `fs::rename` old → new and push a message
3. Read `CLAUDE.md` if it exists; replace `@.apm/agents.md` → `@.apm/agents/default/agents.md` and `@.apm/style.md` → `@.apm/agents/default/style.md`; write back if changed
4. Read `.apm/config.toml` if it exists; replace old `instructions =` strings with new ones using exact string substitution; write back if changed
5. Read `.apm/workflow.toml` if it exists; replace old `instructions =` strings; write back if changed

Use the same string-replace pattern as the existing CLAUDE.md rewrite in `migrate()`.

#### Step 4 — Update default/workflow.toml

`apm-core/src/default/workflow.toml` — five `instructions =` lines:
- All occurrences of `.apm/apm.spec-writer.md` → `.apm/agents/default/apm.spec-writer.md`
- All occurrences of `.apm/apm.worker.md` → `.apm/agents/default/apm.worker.md`

#### Step 5 — Move project files (this repo)

```
git mv .apm/agents.md        .apm/agents/default/agents.md
git mv .apm/apm.spec-writer.md .apm/agents/default/apm.spec-writer.md
git mv .apm/apm.worker.md    .apm/agents/default/apm.worker.md
git mv .apm/style.md         .apm/agents/default/style.md
```

#### Step 6 — Update project config and workflow (this repo)

`.apm/config.toml` — update three `instructions =` fields to new paths.

`.apm/workflow.toml` — update five `instructions =` fields to new paths (same strings as default/workflow.toml).

#### Step 7 — Update CLAUDE.md (this repo)

Replace:
- `@.apm/agents.md` → `@.apm/agents/default/agents.md`
- `@.apm/style.md` → `@.apm/agents/default/style.md`
- Any prose reference to `.apm/style.md` → `.apm/agents/default/style.md`

#### Step 8 — Update sync tests

`apm-core/tests/worker_md_sync.rs`:
- `default_and_project_apm_worker_md_are_identical`: change both path strings and the panic message from `apm.worker.md` (flat) to `agents/default/apm.worker.md`
- `default_and_project_apm_spec_writer_md_are_identical`: same update for spec-writer
- The `default_and_per_agent_apm_worker_md_are_identical` test (checks `agents/claude/`) is unchanged

`apm-core/tests/spec_writer_md_sync.rs`: already checks `agents/claude/` — no change needed.

#### Step 9 — Verify

`cargo test --workspace` must pass. No `grep` hits for `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`, or `.apm/agents.md` (outside of History/comments) should remain in Rust source files.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T02:41Z | — | new | philippepascal |
| 2026-05-04T02:56Z | new | groomed | philippepascal |
| 2026-05-04T03:05Z | groomed | in_design | philippepascal |
| 2026-05-04T03:09Z | in_design | specd | claude-0503-1200-spec1 |
| 2026-05-04T03:19Z | specd | ready | philippepascal |
| 2026-05-04T03:19Z | ready | in_progress | philippepascal |