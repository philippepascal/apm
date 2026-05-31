+++
id = "4d20ba2f"
title = "Make [workers].default mandatory; remove built-in coder fallback"
state = "ammend"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d20ba2f-make-workers-default-mandatory-remove-bu"
created_at = "2026-05-31T02:58:15.922691Z"
updated_at = "2026-05-31T19:36:04.174456Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463"]
+++

## Spec

### Problem

`apm-core/src/start.rs` has three dispatch functions (`run`, `run_next`, `spawn_next_worker`) and one diagnostic function (`resolve_for_diagnostic`), each ending a worker-profile cascade with `.unwrap_or("claude/coder")` or an explicit `else` branch returning `"claude/coder"`. In `validate.rs`, `configured_agent_names` falls back to `"claude"` and `audit_agent_resolution` falls back to `"claude/coder"` via the same pattern. These literals violate the project rule that agent and role names are configuration, not code, and they silently mask missing config — a project that omits `workers.default` dispatches as if it had set `"claude/coder"`, giving no signal that the field is absent.

The fix is to make `[workers].default` mandatory in `config.toml`: change its type from `Option<String>` to `String` (deserialization fails when the key is present but the field is absent), add a validation error when the field is empty (covers the case where `[workers]` is absent entirely and `WorkersConfig::default()` supplies an empty string), and remove every hardcoded `"claude/coder"` fallback from dispatch and validation code. `apm init` already writes the field; no scaffold change is needed.

### Acceptance criteria

- [ ] A `config.toml` that has a `[workers]` section but no `default` key fails TOML deserialization with a clear error.
- [ ] A `config.toml` with no `[workers]` section (default = "") fails `apm validate` with an error that names `config.toml` and the field.
- [ ] A `config.toml` with `[workers] default = ""` (explicitly empty) fails `apm validate` with the same error.
- [ ] A `config.toml` with `[workers] default = "claude/coder"` passes `apm validate` with no error about `workers.default`.
- [ ] `apm start --spawn` on a ticket in a project whose `workers.default` is `"mock-happy/coder"` dispatches using `"mock-happy/coder"`, not `"claude/coder"`.
- [ ] A ripgrep for the literal `claude/coder` in `apm-core/src/` (excluding `src/init.rs`, `src/default/`, test fixture strings, and test TOML snippets that *set* the field) returns no matches.
- [ ] All existing `cargo test --workspace` tests pass.

### Out of scope

- Schema changes to `apm.toml` or other config sections (covered by earlier tickets in this epic).
- Changes to the worker command list or help text.
- Migrating existing user `config.toml` files that lack `[workers].default` — migration path is documented in the problem statement: run `apm init` or add the line manually.
- Removing `DEFAULT_CODER_DEFAULT` and related `include_str!` constants from `start.rs` — they serve `resolve_builtin_instructions()`, which provides default role instruction files and is unrelated to the `workers.default` cascade.

### Approach

#### `apm-core/src/config.rs` — type change

`WorkersConfig.default` is currently `pub default: Option<String>` (line 114). The struct derives `Default` (line 102) via `#[derive(Default)]`.

- Change the field type: `pub default: Option<String>` → `pub default: String`.
- Remove `#[derive(Default)]` from `WorkersConfig`.
- Add a manual `impl Default for WorkersConfig` that sets `default: String::new()` (empty string — caught by the new validation check below) and leaves the other fields as before.
- With this change, when `[workers]` is present but `default` is absent, serde returns a deserialization error. When `[workers]` is absent, `Config.workers` uses `WorkersConfig::default()` (empty string), caught by validation.

Tests to update in `config.rs`:
- `workers_config_default`: change `assert!(config.workers.default.is_none())` → `assert!(config.workers.default.is_empty())`.
- `workers_config_default_field`: change `assert_eq!(config.workers.default.as_deref(), Some("claude/coder"))` → `assert_eq!(config.workers.default, "claude/coder")`.
- Add a new test `workers_default_missing_fails_parse` that asserts `toml::from_str::<Config>(toml_with_workers_but_no_default)` returns `Err`.

#### `apm-core/src/start.rs` — four dispatch sites

Each site follows the same pattern. Replace `.or(config.workers.default.as_deref()).unwrap_or("claude/coder")` with `.unwrap_or(config.workers.default.as_str())`.

**`run()`** (~line 479):
```rust
// Before
let worker_profile_str = triggering_transition
    .and_then(|tr| tr.worker_profile.as_deref())
    .or(config.workers.default.as_deref())
    .unwrap_or("claude/coder")
    .to_string();
// After
let worker_profile_str = triggering_transition
    .and_then(|tr| tr.worker_profile.as_deref())
    .unwrap_or(config.workers.default.as_str())
    .to_string();
```

**`run_next()`** (~line 605) and **`spawn_next_worker()`** (~line 784): identical substitution.

**`resolve_for_diagnostic()`** (~line 180): replace the three-arm `if/else if/else` with a two-arm form:
```rust
// Before (three arms, last arm hardcodes "claude/coder")
let (worker_profile_str, profile_source) = if let Some(wp) = wp_from_transition {
    (wp, format!("workflow.toml transition {transition_label}"))
} else if let Some(default) = &config.workers.default {
    (default.clone(), "workers.default".to_string())
} else {
    ("claude/coder".to_string(), "built-in fallback".to_string())
};
// After
let (worker_profile_str, profile_source) = if let Some(wp) = wp_from_transition {
    (wp, format!("workflow.toml transition {transition_label}"))
} else {
    (config.workers.default.clone(), "workers.default".to_string())
};
```

The `include_str!` constants at the top of the file are not touched — they are used by `resolve_builtin_instructions()` for the role-file cascade, which is separate from the `workers.default` dispatch cascade.

#### `apm-core/src/validate.rs` — three sites

**`validate_config_no_agents()`**: add near the top of the function (before the state-level checks):
```rust
if config.workers.default.is_empty() {
    errors.push(
        "config: workers.default is not set; add `default = \"<agent/role>\"` \
         under [workers] in .apm/config.toml".into()
    );
}
```

**`configured_agent_names()`** (~line 143): the current code uses `as_deref().and_then(...).unwrap_or_else(|| "claude".to_string())`. Replace with a conditional insert:
```rust
if let Some((agent, _)) = config.workers.default.split_once('/') {
    names.insert(agent.to_string());
}
```
This correctly handles an empty `default` (no insert, no fallback).

**`audit_agent_resolution()`** (~line 739):
```rust
// Before
let default_profile = config.workers.default.as_deref().unwrap_or("claude/coder");
// After
let default_profile = config.workers.default.as_str();
```

Tests to update in `validate.rs`:
- `setup_verify_repo()`: add `[workers]\ndefault = "claude/coder"\n` to the config it writes, so all tests that load from that repo see a valid config.
- `correct_config_passes()`: add `[workers]\ndefault = "claude/coder"\n` to its TOML.
- `audit_default_agent_resolution()` and `audit_no_worker_profiles_no_panic()`: add `[workers]\ndefault = "claude/coder"\n` via the extra_toml argument to `audit_config()`.
- Add new tests: `workers_default_absent_fails_validate` (no `[workers]` → error contains "workers.default") and `workers_default_empty_fails_validate` (`default = ""` → same error).

#### `apm-core/src/init.rs` — verify only

`default_config()` (~line 460) already emits `[workers]\ndefault = "{workers_default}"` and the `setup()` function uses `workers_default.unwrap_or("claude/coder")`. No code change needed; the scaffold is already correct.

### Open questions


### Amendment requests

- [ ] Resolve silent panic risk in resolve_for_diagnostic. After removing the unwrap_or claude/coder fallback, the function still calls unwrap on workers.default. Either add an explicit assertion documenting the validation invariant, or use unwrap_or_else with a descriptive error message that names the missing field, so a misconfigured project produces a clear error rather than a confusing crash.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:36Z | groomed | in_design | philippepascal |
| 2026-05-31T07:41Z | in_design | specd | claude |
| 2026-05-31T19:36Z | specd | ammend | philippepascal |