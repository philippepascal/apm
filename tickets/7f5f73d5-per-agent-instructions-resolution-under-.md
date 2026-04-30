+++
id = "7f5f73d5"
title = "Per-agent instructions resolution under .apm/agents/<name>/"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7f5f73d5-per-agent-instructions-resolution-under-"
created_at = "2026-04-30T20:03:33.687625Z"
updated_at = "2026-04-30T21:42:25.580070Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

Each agent may want different prompt conventions (Aider concise context, Codex structured tags, etc.). Move `apm.worker.md` and `apm.spec-writer.md` resolution to be per-agent under `.apm/agents/<name>/`, with project-level overrides retained.

**Reference spec:** `docs/agent-wrappers.md` — section 'Per-agent instructions'.

**Scope:**
- New layout: `.apm/agents/<name>/apm.worker.md` and `.apm/agents/<name>/apm.spec-writer.md` are the per-agent defaults.
- For built-ins: ship the per-agent default markdown bundled in the binary (`include_str!` from `apm-core/src/default/agents/<name>/apm.<role>.md`). For custom wrappers: the user authors them in their wrapper directory.
- Resolution chain (highest priority first), per spawn (profile = P, role = worker|spec-writer, agent = A):
  1. `[worker_profiles.<P>].instructions` (project-level override, full path)
  2. `[workers].instructions` (project-level override, applies to all profiles)
  3. `.apm/agents/<A>/apm.<role>.md` (project-supplied per-agent file, if it exists)
  4. APM's built-in default for agent A (only for built-in agents)
  5. Hard error if none of the above resolve
- The spawn code passes the resolved file path's contents as the system prompt (already happens; just change where the path comes from).
- For migration: existing `.apm/apm.worker.md` and `.apm/apm.spec-writer.md` continue to work because they are referenced by `[workers].instructions` and `[worker_profiles.<P>].instructions` in the default config (project-level overrides at level 1/2). No automatic migration needed — users keep what they have unless they delete the override and want the per-agent default.

**Built-in defaults to ship:**
- `apm-core/src/default/agents/claude/apm.worker.md` — copy of the current default `apm.worker.md`.
- `apm-core/src/default/agents/claude/apm.spec-writer.md` — copy of the current default `apm.spec-writer.md`.
- (Mock built-ins from a separate ticket may not need spec-writer/worker .md files at all; defer to that ticket.)

**Out of scope:**
- Updating the .md content for non-Claude agents — there are no other built-ins yet.
- Per-agent `agents.md` (the project-wide conventions file is still `.apm/agents.md`, not per-agent).
- Sync test extending the existing `apm.worker.md` byte-identical check to other roles — separate concern.

**Tests:**
- Resolution chain test for each level.
- Hard-error test when no instructions resolve.
- Backward-compat: a project with the old config that references `.apm/apm.worker.md` continues to work without edits.

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
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:42Z | groomed | in_design | philippepascal |
