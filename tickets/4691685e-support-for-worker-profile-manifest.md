+++
id = "4691685e"
title = "support for worker_profile manifest"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4691685e-support-for-worker-profile-manifest"
created_at = "2026-05-24T19:18:32.809526Z"
updated_at = "2026-05-24T19:53:01.011096Z"
+++

## Spec

### Problem

APM currently supports a global `[workers]` config in `.apm/config.toml` and a per-machine `local.toml` override, but there is no way to configure properties per worker profile. All profiles (`claude/worker`, `claude/spec-writer`, etc.) share the same `model`, `env`, and `container` values. This means that if a project wants the spec-writer to use a more capable model (e.g., Opus) while keeping the worker on a faster, cheaper one (e.g., Sonnet), there is no supported way to express that.

The fix is to introduce optional per-profile manifest files at `.apm/agents/<agent>/<role>.toml`. When present, these files supply profile-specific overrides for `model` and `env` that take effect at worker spawn time — in `apm start`, `apm work`, and the server's UI dispatcher — without changing any other behaviour.

### Acceptance criteria

- [ ] When `.apm/agents/<agent>/<role>.toml` is absent, `apm start` behaviour is identical to today (no regression)
- [ ] When `model` is set in `<role>.toml`, the worker is spawned with that model, overriding any value in `[workers].model` or `local.toml`
- [ ] When `[env]` entries are set in `<role>.toml`, they are merged into the worker's env, with manifest values winning on key conflicts over `[workers.env]`
- [ ] When `<role>.toml` exists but is malformed TOML, `apm start` returns an error that includes the file path
- [ ] The manifest applies per-profile: `claude/spec-writer` reads `spec-writer.toml`, `claude/worker` reads `worker.toml`; each profile is independent
- [ ] `apm work` (dispatcher) and the server UI dispatcher pick up the same manifest-derived settings as `apm start`

### Out of scope

- Per-profile manifest files for `container` — only `model` and `env` are overridable in this ticket
- `local.toml` override of the profile manifest — profile manifest wins over `local.toml`; a follow-on ticket can layer per-machine > per-profile priority if needed
- `apm validate` coverage for profile manifest files
- `apm prompt` / `explain` showing the manifest-derived model in provenance output
- Schema documentation or JSON Schema generation for the manifest format

### Approach

All changes are in `apm-core/src/start.rs`. No other files need modification.

#### New struct

Add `WorkerProfileManifest` with serde deserialization:

```toml
# .apm/agents/claude/spec-writer.toml
model = "claude-opus-4-5"

[env]
MY_VAR = "value"   # optional
```

```rust
#[derive(serde::Deserialize, Default)]
struct WorkerProfileManifest {
    model: Option<String>,
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
}
```

The struct is private to `start.rs`. No public API change.

#### New functions

`load_profile_manifest(root, agent, role) -> Result<Option<WorkerProfileManifest>>`
- Constructs path `.apm/agents/{agent}/{role}.toml`
- Returns `Ok(None)` if the file is absent
- Returns `Err` (with file path in the message) if the file exists but fails to parse

`apply_profile_manifest(root, wp) -> Result<()>`
- Calls `load_profile_manifest(root, &wp.agent, &wp.role)`
- If a manifest is found:
  - If `manifest.model` is `Some`, sets `wp.model = manifest.model`
  - Merges `manifest.env` into `wp.env`, with manifest values winning on key conflicts

#### Call sites

Insert `apply_profile_manifest(root, &mut wp)?;` immediately after each of the three existing `resolve_worker_profile` calls in `start.rs`:
- Line ~291 in `run()`
- Line ~458 in `run_next()`
- Line ~628 in `spawn_next_worker()`

The call in `prompt.rs` (`resolve_agent_role`) is not updated — it only uses `wp.agent` and `wp.role` for prompt inspection; model and env are irrelevant there.

#### Priority chain for model

From lowest to highest: `config.toml [workers].model` → `local.toml [workers].model` (already merged into `config.workers.model` by `Config::load`) → profile manifest `{role}.toml`. The manifest wins over both global config and local machine config.

#### Tests (inline in `start.rs`)

- `load_profile_manifest_returns_none_when_absent` — no file present → `Ok(None)`
- `load_profile_manifest_parses_model` — valid TOML with `model` → model returned
- `load_profile_manifest_errors_on_malformed_toml` — invalid TOML → `Err` containing file path
- `apply_profile_manifest_overrides_model` — manifest model beats `wp.model` set from global config
- `apply_profile_manifest_noop_when_absent` — missing file leaves `wp` unchanged
- `apply_profile_manifest_merges_env_and_manifest_wins_on_conflict` — env keys from manifest override matching keys from global workers env

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T19:18Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
| 2026-05-24T19:53Z | groomed | in_design | philippepascal |