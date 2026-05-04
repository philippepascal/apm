+++
id = "6803b88b"
title = "Decouple instructions from worker_profiles; move to workflow transitions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6803b88b-decouple-instructions-from-worker-profil"
created_at = "2026-05-04T16:48:29.472278Z"
updated_at = "2026-05-04T16:50:37.372051Z"
epic = "5acea599"
target_branch = "epic/5acea599-flexible-agent-configuration"
+++

## Spec

### Problem

Spawning a worker agent for a workflow phase currently requires two coordinated edits in two different files. The transition in `workflow.toml` sets `profile = "spec_agent"`, and the profile in `config.toml` carries `instructions` (the system-prompt file path) and `role_prefix`. The profile was originally introduced for infrastructure overrides (agent binary, model, container), but `instructions` and `role_prefix` are workflow-level concerns: they describe what role the agent plays during a particular phase, not how it is executed.

This coupling has two practical downsides. First, editing which instructions a spec-writer receives requires touching `config.toml`, not `workflow.toml`, where all other workflow-phase behaviour lives. Second, a project that wants distinct instructions per transition but identical infrastructure must create one profile entry per transition, inflating `config.toml` with boilerplate that adds no infrastructure value.

The desired state is that `instructions` and `role_prefix` can be set directly on a `[[workflow.states.transitions]]` block in `workflow.toml`. Projects that need only a role change, without any infrastructure override, would no longer need a `[worker_profiles.*]` entry at all. Projects that need both can continue using a profile; transition-level fields simply take precedence.

### Acceptance criteria

- [ ] `TransitionConfig` in `apm-core/src/config.rs` gains an `instructions: Option<String>` field
- [ ] `TransitionConfig` gains a `role_prefix: Option<String>` field
- [ ] When `transition.instructions` is set, it is used as the worker system prompt in place of `profile.instructions`
- [ ] When `transition.role_prefix` is set, it is used as the worker identity prefix in place of `profile.role_prefix`
- [ ] Resolution order for system prompt is: transition.instructions → profile.instructions → workers.instructions → `.apm/agents/<agent>/apm.<role>.md` → built-in default
- [ ] Resolution order for role prefix is: transition.role_prefix → profile.role_prefix → built-in default string
- [ ] A transition with only `instructions` and `role_prefix` (no `profile`) spawns a worker correctly with the specified system prompt and identity prefix
- [ ] A transition with both `profile` and `instructions` uses `instructions` for the system prompt, ignoring `profile.instructions`
- [ ] A transition with both `profile` and `role_prefix` uses `role_prefix` for the identity prefix, ignoring `profile.role_prefix`
- [ ] `apm start` and `apm next --spawn` both apply the same resolution order
- [ ] `.apm/workflow.toml` is updated: `groomed → in_design` and `ammend → in_design` transitions carry `instructions` and `role_prefix` directly
- [ ] `.apm/workflow.toml` is updated: `ready → in_progress` transition carries `instructions` and `role_prefix` directly
- [ ] `.apm/config.toml` is updated: `[worker_profiles.spec_agent]` and `[worker_profiles.impl_agent]` drop `instructions` and `role_prefix`; profiles that become empty are removed
- [ ] Existing tests for instruction resolution pass without modification (backward compat with profile-only config)

### Out of scope

- Moving other profile fields (model, agent, container, env, options) to transitions — those remain infrastructure concerns owned by worker_profiles\n- Changing the semantics of StateConfig.instructions, which is a user-message prefix used in non-spawn mode (unrelated to the system prompt)\n- Adding instructions/role_prefix to states (as opposed to transitions) — states already have instructions for a different purpose\n- Removing the worker_profiles mechanism; it stays as the path for infra-only overrides\n- Schema validation that a transition references a valid profile name (a separate concern)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:50Z | groomed | in_design | philippepascal |