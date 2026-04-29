+++
id = "163e0ee3"
title = "explore: claude arguments are all in config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/163e0ee3-explore-claude-arguments-are-all-in-conf"
created_at = "2026-04-29T06:58:36.282134Z"
updated_at = "2026-04-29T21:27:31.868020Z"
+++

## Spec

### Problem

In `start.rs`, three Claude-specific invocation details are hardcoded in both `spawn_container_worker()` and `build_spawn_command()`:

1. `--output-format stream-json` (lines 156, 195) — the structured-output flag APM relies on for log capture
2. `--verbose` (lines 161, 200) — required by the Claude CLI whenever `--print` and `--output-format=stream-json` are combined
3. The flag name `--system-prompt` (lines 162, 201) — the flag used to hand the worker its system instructions
4. The flag name `--dangerously-skip-permissions` (lines 163-165, 202-204) — injected conditionally when `skip_permissions` is true

Meanwhile the `WorkersConfig` struct already supports `command`, `args`, `model`, and `env` as configurable fields, and per-profile overrides exist via `WorkerProfileConfig`. The mismatch means a user who wants to swap in a different agent binary (e.g. `aider`, a custom wrapper) cannot do so via config alone — the hardcoded Claude flags will break any non-Claude invocation.

The desired state: every argument that APM appends to the worker command is either (a) already in the user-controlled `args` array, or (b) driven by a named config field with a sensible default. No Claude-specific string should be hard-wired in `start.rs`.

### Acceptance criteria

- [ ] `--output-format stream-json` is no longer hardcoded in `start.rs`; it is part of the resolved `args` list (present in the default config)
- [ ] `--verbose` is no longer hardcoded in `start.rs`; it is part of the resolved `args` list (present in the default config)
- [ ] A `system_prompt_flag` field in `WorkersConfig` controls the flag name used to pass the system prompt; it defaults to `"--system-prompt"`
- [ ] When `system_prompt_flag` is `null` / absent, no system-prompt argument is appended to the command
- [ ] A `skip_permissions_flag` field in `WorkersConfig` controls the flag name appended when permission-skipping is requested; it defaults to `"--dangerously-skip-permissions"`
- [ ] When `skip_permissions_flag` is `null` / absent, no flag is appended even when `apm start --skip-permissions` is used
- [ ] `WorkerProfileConfig` gains the same two fields and they override the global values when set
- [ ] `check_output_format_supported()` is only invoked when `--output-format` appears in the final resolved `args` list
- [ ] Existing projects whose configs do not set the new fields behave identically to today (backward-compatible via defaults)
- [ ] `.apm/config.toml` `[workers]` and all `[worker_profiles.*]` entries explicitly include `--output-format`, `stream-json`, and `--verbose` in their `args` arrays

### Out of scope

- Validating or testing non-Claude agents end-to-end (this ticket makes the config expressive; wiring up a real alternative agent is a follow-up)
- Adding `local.toml` override support for the new `system_prompt_flag` / `skip_permissions_flag` fields
- Changing how the positional ticket-content argument is passed (still always the final positional arg)
- Changing how `--model` is passed (already configurable, unchanged)
- Supporting output-format capture strategies other than `stream-json`
- Changing log parsing logic to handle non-JSON transcript formats

### Approach

Changes span `apm-core/src/config.rs`, `apm-core/src/start.rs`, and `.apm/config.toml`.

**1. `config.rs` — `WorkersConfig`**
- Add two new fields with serde defaults:
  - `system_prompt_flag: Option<String>` — default `Some("--system-prompt")`
  - `skip_permissions_flag: Option<String>` — default `Some("--dangerously-skip-permissions")`
  Add corresponding `default_system_prompt_flag()` and `default_skip_permissions_flag()` functions, and update `WorkersConfig::default()`.
- Change `default_args()` from `["--print"]` to `["--print", "--output-format", "stream-json", "--verbose"]` so new projects get the correct defaults without having to enumerate every flag.

**2. `config.rs` — `WorkerProfileConfig`**
- Add the same two fields as bare `Option<String>` with no serde default (i.e. absent from TOML = `None` = inherit from global). This preserves the existing profile-override pattern.

**3. `start.rs` — `EffectiveWorkerParams`**
- Add `system_prompt_flag: Option<String>` and `skip_permissions_flag: Option<String>` fields.

**4. `start.rs` — `effective_spawn_params()`**
- Resolve both new fields using the same profile-over-global pattern as `command`/`model`: profile value wins if `Some`, otherwise fall back to global `workers` value.

**5. `start.rs` — `spawn_container_worker()` and `build_spawn_command()`**
- Remove the two hardcoded lines for `--output-format stream-json` and `--verbose` — these will now arrive via `params.args`.
- Replace hardcoded `cmd.args(["--system-prompt", worker_system])` with a conditional that only appends the flag+value when `params.system_prompt_flag` is `Some`.
- Replace hardcoded `cmd.arg("--dangerously-skip-permissions")` with a conditional that checks both `skip_permissions == true` and `params.skip_permissions_flag.is_some()`.

**6. `start.rs` — `check_output_format_supported()`**
- Gate the call: only invoke it when `"--output-format"` appears in `params.args`. This avoids a spurious compatibility error for agents that use a different log-capture mechanism.

**7. `.apm/config.toml`**
- Update `[workers].args` and every `[worker_profiles.*].args` to add `"--output-format"`, `"stream-json"`, and `"--verbose"`. The existing config has explicit `args` entries that won't change just because `default_args()` changed, so this manual update is required to keep behaviour identical.

**8. Tests in `config.rs`**
- Update any assertion that `default_args() == ["--print"]` to reflect the new four-element default.
- Add a test for `effective_spawn_params` verifying that `system_prompt_flag` and `skip_permissions_flag` resolve correctly under (a) profile override, (b) fallback to global, and (c) explicit `None` (no flag appended).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-29T06:58Z | — | new | philippepascal |
| 2026-04-29T21:13Z | new | groomed | philippepascal |
| 2026-04-29T21:27Z | groomed | in_design | philippepascal |