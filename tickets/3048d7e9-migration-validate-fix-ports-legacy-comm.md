+++
id = "3048d7e9"
title = "Migration: validate --fix ports legacy command/args/model to agent + options"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3048d7e9-migration-validate-fix-ports-legacy-comm"
created_at = "2026-04-30T20:03:17.277300Z"
updated_at = "2026-04-30T20:03:17.277300Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["6cac8518"]
+++

## Spec

### Problem

When existing projects upgrade APM, their `.apm/config.toml` still uses the legacy `[workers] command/args/model` shape. Provide an automated migration so users do not have to hand-edit. The legacy fields are read with a deprecation warning (per ticket 6cac8518); this ticket adds the migration that retires them.

**Reference spec:** `docs/agent-wrappers.md` — section 'Migration from current config'.

**Scope:**
- Extend `apm validate --fix` to detect a config with legacy fields and rewrite to the new shape:
  - `command = "claude"` → `agent = "claude"`
  - `model = "sonnet"` → `[workers.options] model = "sonnet"`
  - `args = ["--print", "--output-format=stream-json", "--verbose"]` (or any subset) → dropped (the wrapper handles flags)
  - Same migration for every `[worker_profiles.<X>]` section.
- TOML rewrite must preserve comments, ordering of unrelated sections, and trailing whitespace as much as possible (use a TOML-aware editor, e.g. `toml_edit`).
- If `command` is anything other than `claude`, do not auto-migrate — print a warning that the user must hand-pick a wrapper for their custom command and stop.
- After migration, re-run validate to confirm the new config parses cleanly. The old fields should be entirely gone (no commented-out leftovers).
- Add a one-line migration message: `migrated [workers] config to agent-driven shape; legacy command/args/model removed`.

**Out of scope:**
- An interactive `apm init --migrate` subcommand. Validate --fix is the canonical migration path.
- Migration of any `.apm/agents.md` or `.apm/apm.*.md` files. Those are content, not config.
- Hash-trip integration changes — the existing hash-trip already runs validate on config change.

**Tests:**
- Repo with legacy config + claude command → fix produces the new shape; re-validate passes.
- Repo with legacy config + non-claude command → fix prints a warning and does not modify config.
- Repo with mixed legacy + new fields → fix removes legacy fields, preserves new ones.
- Repo with already-migrated config → fix is a no-op.
- TOML preservation: a config with a comment between sections survives the fix unchanged.

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
