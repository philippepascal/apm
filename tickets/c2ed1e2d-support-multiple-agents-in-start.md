+++
id = "c2ed1e2d"
title = "support multiple agents in start"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
branch = "ticket/c2ed1e2d-support-multiple-agents-in-start"
created_at = "2026-04-07T17:14:02.689742Z"
updated_at = "2026-04-07T18:12:15.007216Z"
+++

## Spec

### Problem

Currently, `apm start`, `apm work`, and the UI work engine all share a single global worker configuration (`[workers]` in `config.toml`). The only differentiation between agent roles is hardcoded in `start.rs`: a static array `["groomed", "ammend"]` controls whether the spec-writer system prompt (`.apm/apm.spec-writer.md`) or the default worker prompt (`.apm/apm.worker.md`) is injected — but the spawn command, args, model, and environment are identical for both roles.

This makes it impossible to run a different binary, model, or container image for spec-writing vs. implementation, and impossible to add new agent roles (e.g. a QA agent, a review agent) without modifying Rust source code.

The desired behaviour is a named **worker profile** system. Users define profiles (e.g. `spec_agent`, `impl_agent`) in config, each with its own spawn command, args, model, environment, and instructions file. States in `workflow.toml` reference a profile by name. When `apm start` (or `apm work`) picks up a ticket, it reads the pre-transition state's profile and spawns the correct agent type automatically. Adding a new agent role becomes a config-only change.

### Acceptance criteria

- [ ] A `[worker_profiles.<name>]` table can be defined in `.apm/config.toml`; `apm` loads it without error
- [ ] `WorkerProfileConfig` supports optional fields: `command`, `args`, `model`, `env`, `container`, `instructions`, `role_prefix`
- [ ] A transition in `workflow.toml` can declare `profile = "<name>"`; `apm` loads the workflow without error
- [ ] When `apm start` is called on a ticket and the triggering transition has `profile = "spec_agent"`, the `spec_agent` profile's `instructions` file is used as the system prompt
- [ ] When `apm start` is called on a ticket and the triggering transition has `profile = "impl_agent"`, the `impl_agent` profile's `instructions` file is used as the system prompt
- [ ] A profile's `command` overrides the global `[workers].command` for spawned subprocesses
- [ ] A profile's `model` overrides the global `[workers].model` for spawned subprocesses
- [ ] A profile's `env` is merged on top of the global `[workers].env` for spawned subprocesses
- [ ] A profile's `container` overrides the global `[workers].container` for spawned subprocesses
- [ ] Profile fields that are absent fall back to the corresponding global `[workers]` value
- [ ] When a transition has no `profile` field, the global `[workers]` config is used and the state's `instructions` file (or `.apm/apm.worker.md` fallback) is the system prompt (existing behaviour preserved)
- [ ] When a transition references a profile name that is not defined in config, `apm start` falls back to global `[workers]` config and prints a warning
- [ ] `apm work` dispatches each worker using the profile of its ticket's triggering transition
- [ ] The hardcoded `spec_writer_states` array `["groomed", "ammend"]` is removed from `start.rs`
- [ ] The project's own `.apm/workflow.toml` declares `profile = "spec_agent"` on the `groomed → in_design` and `ammend → in_design` transitions
- [ ] The project's own `.apm/workflow.toml` declares `profile = "impl_agent"` on the `ready → in_progress` transition
- [ ] The project's own `.apm/config.toml` defines `[worker_profiles.spec_agent]` and `[worker_profiles.impl_agent]` with their respective `instructions` paths

### Out of scope

- Per-profile `keychain` (API key) overrides — keychain stays global
- Per-profile `skip_permissions` — remains a global flag on `AgentsConfig`
- A `--profile` CLI flag to override the profile at invocation time
- Profile overrides in `local.toml` (local per-machine profile overrides)
- UI surface for editing or selecting profiles
- Profile inheritance / composition (profiles extending other profiles)
- Hot-reloading profiles without restarting `apm work`

### Approach

**1. New config struct — `WorkerProfileConfig` (`apm-core/src/config.rs`)**

Add a new struct, all fields optional (absent = fall back to global `[workers]`):

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WorkerProfileConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub container: Option<String>,
    pub instructions: Option<String>,   // path to system-prompt file
    pub role_prefix: Option<String>,    // "You are a Spec-Writer agent assigned to ticket #<id>."
}
```

Add `#[serde(default)] pub worker_profiles: HashMap<String, WorkerProfileConfig>` to `Config`.

TOML surface in `.apm/config.toml`:
```toml
[worker_profiles.spec_agent]
instructions = ".apm/apm.spec-writer.md"
role_prefix = "You are a Spec-Writer agent assigned to ticket #<id>."

[worker_profiles.impl_agent]
instructions = ".apm/apm.worker.md"
role_prefix = "You are a Worker agent assigned to ticket #<id>."
```

**2. Profile field on `TransitionConfig` (`apm-core/src/config.rs`)**

Add `#[serde(default)] pub profile: Option<String>` to `TransitionConfig`, not `StateConfig`. The profile determines which agent binary/config to use at spawn time, and spawning is triggered by a transition.

TOML surface in `.apm/workflow.toml`:
```toml
[[workflow.states.transitions]]
to      = "in_design"
trigger = "command:start"
actor   = "agent"
profile = "spec_agent"
```

**3. Profile resolution helpers in `start.rs`**

- `resolve_profile<'a>(transition: &TransitionConfig, config: &'a Config) -> Option<&'a WorkerProfileConfig>`: if `transition.profile` is `Some(name)`, looks up `config.worker_profiles[name]`; if the name is set but not found, logs a warning and returns `None`.
- `effective_spawn_params(profile: Option<&WorkerProfileConfig>, workers: &WorkersConfig) -> EffectiveWorkerParams`: merges profile fields over global workers — `command`, `args`, `model`, `env` (profile env merged on top of workers env), `container` all fall back to `workers.*` if absent in profile.
- `resolve_system_prompt`: reads `profile.instructions` path, falling back to the state's existing `instructions` field, then `.apm/apm.worker.md`.
- `agent_role_prefix`: reads `profile.role_prefix` with `<id>` substituted, falling back to the default "Worker agent" prefix.
- Remove the `spec_writer_states: ["groomed", "ammend"]` static array entirely.

**4. Thread effective params into spawn functions**

`build_spawn_command` and `spawn_container_worker` currently read `command`, `args`, `model`, `container`, `env` directly from `&WorkersConfig`. Introduce an `EffectiveWorkerParams` struct (or update their signatures) so they use the merged values instead. Profile resolution runs once in `start_worker_in_worktree` before either spawn path is entered.

**5. Update project config files**

- `.apm/workflow.toml`: add `profile = "spec_agent"` to the `groomed → in_design` and `ammend → in_design` transition tables; add `profile = "impl_agent"` to the `ready → in_progress` transition table. The existing `instructions` fields on states remain as fallbacks.
- `.apm/config.toml`: add `[worker_profiles.spec_agent]` and `[worker_profiles.impl_agent]` sections pointing to the existing `.apm/apm.spec-writer.md` and `.apm/apm.worker.md` files.

**Order of steps**

1. Add `WorkerProfileConfig` + `worker_profiles` to `Config`; add `profile` to `TransitionConfig`; compile-check only.
2. Add `resolve_profile` + `effective_spawn_params` helpers in `start.rs`.
3. Thread effective params into spawn functions; remove `spec_writer_states` hardcode.
4. Update `.apm/config.toml` and `.apm/workflow.toml`.
5. Run existing tests; add unit tests for `resolve_profile` (missing profile falls back with warning) and `effective_spawn_params` (merge precedence).

**Constraints**

- Backward-compatible: transitions without `profile` fall through to state `instructions` field then `.apm/apm.worker.md` (no behaviour change for existing projects).
- `keychain` stays on `WorkersConfig`, not on profiles.
- No HTTP API or UI changes required.

### 1. New config struct — `WorkerProfileConfig` (`apm-core/src/config.rs`)

Add a new struct, all fields optional (absent = fall back to global `[workers]`):

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WorkerProfileConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub container: Option<String>,
    pub instructions: Option<String>,   // path to system-prompt file
    pub role_prefix: Option<String>,    // e.g. "You are a Spec-Writer agent assigned to ticket #<id>."
}
```

Add to `Config`:
```rust
#[serde(default)]
pub worker_profiles: HashMap<String, WorkerProfileConfig>,
```

TOML surface:
```toml
[worker_profiles.spec_agent]
instructions = ".apm/apm.spec-writer.md"
role_prefix = "You are a Spec-Writer agent assigned to ticket #<id>."

[worker_profiles.impl_agent]
instructions = ".apm/apm.worker.md"
role_prefix = "You are a Worker agent assigned to ticket #<id>."
```

(`<id>` is replaced at runtime with the ticket ID, matching the existing interpolation pattern.)

### 2. Profile field on `StateConfig` (`apm-core/src/config.rs`)

```rust
#[serde(default)]
pub profile: Option<String>,
```

### 3. Profile resolution in `start.rs`

Add `resolve_profile<'a>(state_id: &str, config: &'a Config) -> Option<&'a WorkerProfileConfig>`:
- Look up `StateConfig` for `state_id`
- If `state.profile` is `Some(name)`, look up `config.worker_profiles[name]`; if missing, warn and return `None`
- If `state.profile` is `None`, return `None`

Add `effective_spawn_params(profile: Option<&WorkerProfileConfig>, workers: &WorkersConfig)` that merges profile over globals:
- `command` = profile.command falling back to workers.command
- `args` = profile.args falling back to workers.args
- `model` = profile.model falling back to workers.model
- `env` = workers.env merged with profile.env (profile wins on collision)
- `container` = profile.container falling back to workers.container

Replace `resolve_system_prompt` with a lookup of `profile.instructions` path (falling back to `.apm/apm.worker.md`).

Replace `agent_role_prefix` with `profile.role_prefix` with `<id>` substituted (falling back to `"You are a Worker agent assigned to ticket #<id>."`).

Remove the `spec_writer_states` static array entirely.

### 4. Thread effective params into spawn functions

`build_spawn_command` and `spawn_container_worker` currently read directly from `config.workers`. Introduce an `EffectiveWorkerParams` struct (or pass individual values) derived from the merge above, and update both functions to accept it instead of `&WorkersConfig` directly.

Both functions are called from `start_worker_in_worktree`. Profile resolution happens once there, before either spawn path is taken.

### 5. Update project config files

`.apm/workflow.toml` — add `profile = "spec_agent"` to `groomed` and `ammend` state tables; add `profile = "impl_agent"` to `ready`.

`.apm/config.toml` — add `[worker_profiles.spec_agent]` and `[worker_profiles.impl_agent]` sections pointing to the existing instruction files.

### Order of steps

1. Add `WorkerProfileConfig` and `worker_profiles` to `Config`; add `profile` to `StateConfig`; update deserialization — compile-check only.
2. Add `resolve_profile` and `effective_spawn_params` helpers in `start.rs`.
3. Thread effective params into `build_spawn_command` / `spawn_container_worker`; remove `spec_writer_states` hardcode.
4. Update `.apm/config.toml` and `.apm/workflow.toml`.
5. Run existing test suite; add unit tests for `resolve_profile` (missing profile warns + falls back) and `effective_spawn_params` (merge precedence).

### Constraints

- Must remain backward-compatible: projects without `profile` on their states get the old behaviour (global workers config + `.apm/apm.worker.md`).
- `keychain` stays on `WorkersConfig`, not on profiles.
- No changes to the HTTP API or UI are required for this ticket.

### Open questions


### Amendment requests

- [x] Move `profile` from `StateConfig` to the transition definition. The profile determines which agent to spawn, and spawning happens on the transition (e.g. groomed→in_design, ready→in_progress), not while sitting in a state. In workflow.toml this would look like: `[[workflow.states.transitions]]` with `profile = "spec_agent"` on the groomed→in_design transition and `profile = "impl_agent"` on the ready→in_progress transition. Update the Approach, AC, and config examples accordingly. The existing `instructions` field on states should be superseded by the profile's `instructions` when a profile is present on the transition.
- [ ] Remove the duplicated sections 1-5 at the bottom of the Approach that repeat the content already covered above

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:14Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:47Z | groomed | in_design | philippepascal |
| 2026-04-07T17:54Z | in_design | specd | claude-0407-1747-3908 |
| 2026-04-07T18:12Z | specd | ammend | claude-0407-review |
| 2026-04-07T18:12Z | ammend | in_design | philippepascal |