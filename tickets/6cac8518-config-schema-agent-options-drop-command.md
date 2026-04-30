+++
id = "6cac8518"
title = "Config schema: agent + options (drop command/args/model)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cac8518-config-schema-agent-options-drop-command"
created_at = "2026-04-30T20:02:34.693415Z"
updated_at = "2026-04-30T21:17:25.961937Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

The wrapper dispatcher landed in d3b93b95 always resolves to the `claude` built-in regardless of config — there is no config-driven dispatch yet. Projects cannot choose their agent, pass a model name cleanly through the new path, or extend behaviour without modifying Rust. Meanwhile, `[workers] command/args/model` are still the authoritative fields even though wrappers now own CLI construction.\n\nThis ticket wires the config to the dispatcher. After it lands, `[workers] agent = "claude"` selects the built-in; `[workers.options]` passes arbitrary key-value pairs that are forwarded to the wrapper as `APM_OPT_<KEY>` env vars. Model selection moves to `options.model`. Legacy `command`, `args`, and `model` fields remain parseable for backward compatibility but no longer drive spawn behaviour; a one-time deprecation warning is emitted to stderr when they are present without the new `agent` field.\n\nThe desired state: a project sets `agent = "claude"` (or omits it to accept the default) and `options.model = "sonnet"`, and the dispatcher calls `resolve_builtin("claude").spawn(ctx)` with `ctx.options` populated — identical runtime behaviour to today, but driven by the new architecture.

### Acceptance criteria

- [ ] `WorkersConfig` deserializes a TOML block containing `agent = "claude"` and `[workers.options]` without error\n- [ ] `WorkerProfileConfig` deserializes a profile block containing `agent` and `options` without error\n- [ ] A config with `workers.agent = "codex"` and `profile.agent` absent resolves the effective agent to `"codex"`\n- [ ] A config with `workers.agent = "codex"` and `profile.agent = "mock-happy"` resolves the effective agent to `"mock-happy"`\n- [ ] A config with neither `workers.agent` nor `profile.agent` set resolves the effective agent to `"claude"`\n- [ ] `profile.options` keys override `workers.options` keys when both define the same key\n- [ ] `profile.options` and `workers.options` keys that do not overlap are both present in the effective options map\n- [ ] Each entry in the effective options map is forwarded as an env var named `APM_OPT_<KEY>` (key uppercased, dots and dashes replaced with underscores)\n- [ ] `options.model = "sonnet"` results in `APM_OPT_MODEL=sonnet` being set on the spawned child\n- [ ] A config using only legacy `command = "claude"` (no `agent` field) still spawns the claude wrapper successfully\n- [ ] When legacy `command`, `args`, or `model` fields are present and `agent` is absent, exactly one line is written to stderr containing the word `deprecated` per process run\n- [ ] The deprecation warning is not emitted a second time if a second worker is spawned in the same process\n- [ ] Legacy `model = "sonnet"` with no `options.model` still produces the correct `--model sonnet` flag in the spawned claude command\n- [ ] `apm init` generates a config with `agent = "claude"`, `options.model = "sonnet"`, and no `command` or `args` fields\n- [ ] A config with no `[workers]` section at all spawns successfully with defaults (agent = claude)

### Out of scope

- Custom wrapper resolution from `.apm/agents/<name>/` — ticket 2c32a282\n- Per-ticket frontmatter `agent` / `agent_overrides` override — ticket 0ca3e019\n- `apm migrate --fix` automated config file rewrite — ticket 3048d7e9\n- Mock wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — ticket 25c92daa\n- Removing `check_output_format_supported()` — deferred to wrapper-versioning ticket 2e772eab\n- Wrapper-contract versioning checks against `manifest.toml` — ticket 2e772eab\n- Per-agent instruction file resolution under `.apm/agents/<name>/` — ticket 7f5f73d5\n- The `apm agents` subcommand — ticket 71d80e40

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
| 2026-04-30T21:17Z | groomed | in_design | philippepascal |