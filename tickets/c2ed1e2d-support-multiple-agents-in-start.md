+++
id = "c2ed1e2d"
title = "support multiple agents in start"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c2ed1e2d-support-multiple-agents-in-start"
created_at = "2026-04-07T17:14:02.689742Z"
updated_at = "2026-04-07T17:47:54.483986Z"
+++

## Spec

### Problem

Currently, `apm start`, `apm work`, and the UI work engine all share a single global worker configuration (`[workers]` in `config.toml`). The only differentiation between agent roles is hardcoded in `start.rs`: a static array `["groomed", "ammend"]` controls whether the spec-writer system prompt (`.apm/apm.spec-writer.md`) or the default worker prompt (`.apm/apm.worker.md`) is injected — but the spawn command, args, model, and environment are identical for both roles.

This makes it impossible to run a different binary, model, or container image for spec-writing vs. implementation, and impossible to add new agent roles (e.g. a QA agent, a review agent) without modifying Rust source code.

The desired behaviour is a named **worker profile** system. Users define profiles (e.g. `spec_agent`, `impl_agent`) in config, each with its own spawn command, args, model, environment, and instructions file. States in `workflow.toml` reference a profile by name. When `apm start` (or `apm work`) picks up a ticket, it reads the pre-transition state's profile and spawns the correct agent type automatically. Adding a new agent role becomes a config-only change.

### Acceptance criteria

- [ ] A `[worker_profiles.<name>]` table can be defined in `.apm/config.toml`; `apm` loads it without error
- [ ] `WorkerProfileConfig` supports optional fields: `command`, `args`, `model`, `env`, `container`, `instructions`, `role_prefix`
- [ ] A state in `workflow.toml` can declare `profile = "<name>"`; `apm` loads the workflow without error
- [ ] When `apm start` is called on a ticket whose pre-transition state has `profile = "spec_agent"`, the `spec_agent` profile's `instructions` file is used as the system prompt
- [ ] When `apm start` is called on a ticket whose pre-transition state has `profile = "impl_agent"`, the `impl_agent` profile's `instructions` file is used as the system prompt
- [ ] A profile's `command` overrides the global `[workers].command` for spawned subprocesses
- [ ] A profile's `model` overrides the global `[workers].model` for spawned subprocesses
- [ ] A profile's `env` is merged on top of the global `[workers].env` for spawned subprocesses
- [ ] A profile's `container` overrides the global `[workers].container` for spawned subprocesses
- [ ] Profile fields that are absent fall back to the corresponding global `[workers]` value
- [ ] When a state has no `profile` field, the global `[workers]` config is used and `.apm/apm.worker.md` is the system prompt (existing behaviour preserved)
- [ ] When a state references a profile name that is not defined in config, `apm start` falls back to global `[workers]` config and prints a warning
- [ ] `apm work` dispatches each worker using the profile of its ticket's pre-transition state
- [ ] The hardcoded `spec_writer_states` array `["groomed", "ammend"]` is removed from `start.rs`
- [ ] The project's own `.apm/workflow.toml` declares `profile = "spec_agent"` on the `groomed` and `ammend` states
- [ ] The project's own `.apm/workflow.toml` declares `profile = "impl_agent"` on the `ready` state
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:14Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:47Z | groomed | in_design | philippepascal |