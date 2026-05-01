+++
id = "6cac8518"
title = "Config schema: agent + options (drop command/args/model)"
state = "specd"
priority = 0
effort = 4
risk = 4
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cac8518-config-schema-agent-options-drop-command"
created_at = "2026-04-30T20:02:34.693415Z"
updated_at = "2026-05-01T01:29:00.875207Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

The wrapper dispatcher landed in d3b93b95 always resolves to the `claude` built-in regardless of config — there is no config-driven dispatch yet. Projects cannot choose their agent, pass a model name cleanly through the new path, or extend behaviour without modifying Rust. Meanwhile, `[workers] command/args/model` are still the authoritative fields even though wrappers now own CLI construction.\n\nThis ticket wires the config to the dispatcher. After it lands, `[workers] agent = "claude"` selects the built-in; `[workers.options]` passes arbitrary key-value pairs that are forwarded to the wrapper as `APM_OPT_<KEY>` env vars. Model selection moves to `options.model`. Legacy `command`, `args`, and `model` fields remain parseable for backward compatibility but no longer drive spawn behaviour; a one-time deprecation warning is emitted to stderr when they are present without the new `agent` field.\n\nThe desired state: a project sets `agent = "claude"` (or omits it to accept the default) and `options.model = "sonnet"`, and the dispatcher calls `resolve_builtin("claude").spawn(ctx)` with `ctx.options` populated — identical runtime behaviour to today, but driven by the new architecture.

### Acceptance criteria

- [ ] `WorkersConfig` deserializes a TOML block containing `agent = "claude"` and `[workers.options]` without error
- [ ] `WorkerProfileConfig` deserializes a profile block containing `agent` and `options` without error
- [ ] A config with `workers.agent = "codex"` and `profile.agent` absent resolves the effective agent to `"codex"`
- [ ] A config with `workers.agent = "codex"` and `profile.agent = "mock-happy"` resolves the effective agent to `"mock-happy"`
- [ ] A config with neither `workers.agent` nor `profile.agent` set resolves the effective agent to `"claude"`
- [ ] `profile.options` keys override `workers.options` keys when both define the same key
- [ ] `profile.options` and `workers.options` keys that do not overlap are both present in the effective options map
- [ ] Each entry in the effective options map is forwarded as an env var named `APM_OPT_<KEY>` (key uppercased, dots and dashes replaced with underscores)
- [ ] `options.model = "sonnet"` results in `APM_OPT_MODEL=sonnet` being set on the spawned child
- [ ] A config using only legacy `command = "claude"` (no `agent` field) still spawns the claude wrapper successfully
- [ ] When legacy `command`, `args`, or `model` fields are present and `agent` is absent, a line containing the substring `deprecated` is written to stderr
- [ ] Across the lifetime of a single process, stderr contains the substring `deprecated` exactly once — even when multiple workers with legacy config are spawned
- [ ] Legacy `model = "sonnet"` with no `options.model` still produces the correct `--model sonnet` flag in the spawned claude command
- [ ] `apm init` generates a config with `agent = "claude"`, `options.model = "sonnet"`, and no `command` or `args` fields
- [ ] A config with no `[workers]` section at all spawns successfully with defaults (agent = claude)

### Out of scope

- Custom wrapper resolution from `.apm/agents/<name>/` — ticket 2c32a282\n- Per-ticket frontmatter `agent` / `agent_overrides` override — ticket 0ca3e019\n- `apm migrate --fix` automated config file rewrite — ticket 3048d7e9\n- Mock wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — ticket 25c92daa\n- Removing `check_output_format_supported()` — deferred to wrapper-versioning ticket 2e772eab\n- Wrapper-contract versioning checks against `manifest.toml` — ticket 2e772eab\n- Per-agent instruction file resolution under `.apm/agents/<name>/` — ticket 7f5f73d5\n- The `apm agents` subcommand — ticket 71d80e40

### Approach

Four files change, plus tests.

### 1. apm-core/src/config.rs

WorkersConfig — add agent: Option<String> (no serde default) and options: HashMap<String,String> with serde(default). Demote command from String-with-serde-default to Option<String> (no default); same for args (Vec<String> with default -> Option<Vec<String>>). Remove default_command() and default_args() free functions and their serde attributes. Update WorkersConfig::default() so command and args are None. The model, env, container, keychain fields are unchanged.

WorkerProfileConfig — add agent: Option<String> and options: HashMap<String,String> with serde(default). All other fields already Option; leave them.

### 2. apm-core/src/start.rs

EffectiveWorkerParams — add agent: String and options: HashMap<String,String>.

effective_spawn_params() additions:

Agent resolution: raw_agent = profile.agent.clone().or_else(|| workers.agent.clone()). If raw_agent is None AND any legacy field (command, args, model at either level) is Some, call emit_deprecation_warning(). Then agent = raw_agent.unwrap_or("claude".to_string()).

Deprecation gate: declare a module-level static AtomicBool (DEPRECATION_WARNED, default false). emit_deprecation_warning() does compare_exchange false->true; only on success does it eprintln the message. This guarantees exactly one emission per process regardless of how many workers are spawned.

Options merge: start from workers.options.clone(), then for each (k,v) in profile.options insert into the map (profile wins on collision).

WrapperContext construction: ctx.options = resolved options map. ctx.model = options.get("model").cloned().or_else(|| params.model.clone()) — this honours both new-style options.model and legacy model field, with new-style winning.

Dispatcher call: resolve_builtin(&params.agent). If None (unknown built-in), return an error with the agent name in the message. Custom-wrapper lookup (ticket 2c32a282) is not part of this ticket; a clear error is sufficient.

### 3. apm-core/src/wrapper/claude.rs (from d3b93b95)

After setting the existing APM contract env vars, add a loop over ctx.options: for each (k, v), compute the env key as "APM_OPT_" + k.to_uppercase() with '.' and '-' replaced by '_', then:
- Local path: cmd.env(env_key, v)
- Container path: push "--env" and "KEY=VAL" as separate docker args

### 4. apm-core/src/init.rs — default_config()

Replace the [workers] block with:
  agent = "claude"
  [workers.options]
  model = "sonnet"

Replace the two [worker_profiles.*] blocks to keep only instructions and role_prefix (no command, args, or model). Profiles inherit [workers] agent and options.

### 5. Tests

- config_round_trip_new_shape: parse TOML with agent + [workers.options], assert fields match
- config_round_trip_legacy_shape: parse TOML with only command/args/model, assert agent is None
- resolution_agent_profile_overrides_global: workers.agent="codex", profile.agent="mock-happy" -> effective="mock-happy"
- resolution_agent_falls_back_to_claude: neither set -> effective="claude"
- resolution_options_merge: workers has {model=opus,timeout=30}, profile has {model=sonnet} -> effective {model=sonnet,timeout=30}
- deprecation_warning_emitted_once: call effective_spawn_params twice with legacy config; capture stderr; assert the captured output contains the substring "deprecated" exactly once
- apm_opt_env_vars_set: mock script writes env to temp file; assert APM_OPT_MODEL=sonnet is present (same pattern as claude_wrapper_sets_apm_env_vars from d3b93b95)
- legacy_model_forwarded_to_ctx: workers.model=Some(opus), options empty -> ctx.model=Some(opus)
- options_model_takes_precedence_over_legacy: workers.model=Some(opus), options.model=sonnet -> ctx.model=Some(sonnet)

### Open questions


### Amendment requests

- [x] The deprecation-warning test AC currently asserts on internal `AtomicBool` state ("check AtomicBool state shows the warning fired exactly once"). This is implementation-aware and brittle — any refactor that changes the gating mechanism breaks the test even if behaviour is correct. Rewrite the AC to assert on captured stderr: "stderr contains the substring `deprecated` exactly once across the process lifetime, regardless of how many times legacy fields are read." The internal `AtomicBool` becomes an implementation detail, not a tested contract.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:17Z | groomed | in_design | philippepascal |
| 2026-04-30T21:23Z | in_design | specd | claude-0430-2117-71a8 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:26Z | ammend | in_design | philippepascal |
| 2026-05-01T01:29Z | in_design | specd | claude-0501-0126-ffc0 |
