+++
id = "6cac8518"
title = "Config schema: agent + options (drop command/args/model)"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cac8518-config-schema-agent-options-drop-command"
created_at = "2026-04-30T20:02:34.693415Z"
updated_at = "2026-04-30T21:02:20.952245Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

Replace the legacy `[workers] command/args/model` config triplet with the wrapper-driven shape: `[workers] agent = "<name>"` plus a `[workers.options]` table for wrapper-specific options. Same for `[worker_profiles.<X>]`. Read `agent` selection from config to drive the wrapper dispatcher landed in d3b93b95.

**Reference spec:** `docs/agent-wrappers.md` — sections 'Configuration', 'Options table'.

**Scope:**
- `apm-core/src/config.rs`:
  - `WorkersConfig`: add `agent: Option<String>` (default `Some("claude")`), `options: HashMap<String, String>` (default empty). Keep `command`, `args`, `model` as deprecated optional fields for backward-compat read (see migration ticket); they no longer drive spawn behaviour.
  - `WorkerProfileConfig`: add same two fields. Profile values override global if set.
- `apm-core/src/start.rs`:
  - Resolve effective agent name: profile → workers → built-in default `claude`.
  - Resolve effective options: profile.options merged over workers.options.
  - Pass agent name to the wrapper dispatcher (built or custom) from d3b93b95.
  - Set `APM_OPT_<KEY>` env vars from the resolved options map (key uppercased, dots/dashes → underscores).
  - When legacy `command/args/model` are present in config and `agent` is absent, synthesize `agent = "claude"` and emit a deprecation warning to stderr (one-time per process). Migration to the new shape lands in the next ticket.
- Update `apm-core/src/default/config.toml` (the init template) to use the new shape: `agent = "claude"`, `options.model = "sonnet"`. Drop `command`/`args`. Same for the two default worker_profiles.

**Out of scope:**
- Custom wrappers from `.apm/agents/<name>/` (separate ticket).
- Frontmatter override (separate ticket).
- Migration helper for existing repos (separate ticket).
- The `check_output_format_supported` removal — can be done here as a side cleanup since wrappers now own their compat checks; or keep until the wrapper-versioning ticket. Pick one in spec phase.

**Tests:**
- Round-trip the new schema through TOML.
- Resolution chain test (profile.agent overrides workers.agent; profile.options overrides workers.options key-by-key).
- Backward-compat test: a config with only legacy `command = "claude"` resolves to the claude wrapper with a deprecation message.

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
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
