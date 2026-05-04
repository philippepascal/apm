+++
id = "ed16b686"
title = "Shell-script wrapper dispatch via .apm/agents/<name>/wrapper.sh"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ed16b686-shell-script-wrapper-dispatch-via-apm-ag"
created_at = "2026-05-04T16:48:38.984881Z"
updated_at = "2026-05-04T16:56:05.548682Z"
epic = "5acea599"
target_branch = "epic/5acea599-flexible-agent-configuration"
depends_on = ["6803b88b"]
+++

## Spec

### Problem

Two gaps prevent per-transition agent dispatch from working after ticket 6803b88b lands.

First, `TransitionConfig` has no `agent` field. Before 6803b88b, the agent name came from `[worker_profiles.<name>].agent`; that profile mechanism is being removed for spec/impl agents. Without `transition.agent`, every spawn falls back to the built-in `"claude"` regardless of what custom wrapper scripts are installed in `.apm/agents/`.

Second, `.apm/agents/default/` — the directory whose instruction files are already referenced by the post-6803b88b transitions — has no `wrapper.sh`. Even if `transition.agent = "default"` were wired up, there would be nothing to dispatch to: the resolver would find no built-in named "default" and bail with an error.

The existing wrapper dispatch infrastructure in `apm-core/src/wrapper/` already discovers and executes shell scripts at `.apm/agents/<name>/wrapper.*`. This ticket surfaces that capability at the configuration layer by adding `agent` to `TransitionConfig`, threading it through `effective_spawn_params()`, and providing `.apm/agents/default/wrapper.sh` — a project-editable script that reproduces the built-in claude invocation using the environment variables custom wrappers already receive. A model env-var gap in the custom-wrapper env setup is also patched so `wrapper.sh` can honour `--model` overrides without hard-coding the value.

### Acceptance criteria

- [ ] `TransitionConfig` in `apm-core/src/config.rs` exposes `agent: Option<String>` with `#[serde(default)]`
- [ ] `effective_spawn_params()` accepts `transition_agent: Option<&str>` as its first argument and checks it before `profile.agent` and `workers.agent`
- [ ] When `transition.agent = "default"`, `spawn_worker()` dispatches through `.apm/agents/default/wrapper.sh`
- [ ] A `wrapper.sh` in `.apm/agents/<name>/` with no accompanying `manifest.toml` is accepted without error (manifest defaults apply)
- [ ] `.apm/agents/default/wrapper.sh` exists and is executable (`chmod +x`)
- [ ] `wrapper.sh` invokes the claude binary with `--print --output-format stream-json --verbose --disable-slash-commands`, reading the system prompt from `$APM_SYSTEM_PROMPT_FILE` and the user message from `$APM_USER_MESSAGE_FILE`
- [ ] `wrapper.sh` passes `--dangerously-skip-permissions` when `APM_SKIP_PERMISSIONS` is set
- [ ] `wrapper.sh` passes `--model "$APM_MODEL"` when `APM_MODEL` is non-empty
- [ ] The env-var setup for custom wrapper scripts includes `APM_MODEL` (set from `ctx.model`)
- [ ] The `groomed → in_design`, `ammend → in_design`, and `ready → in_progress` transitions in `.apm/workflow.toml` each include `agent = "default"`
- [ ] When no `transition.agent`, no `profile.agent`, and no `workers.agent` are set, the resolved agent name is `"claude"` (unchanged behaviour)
- [ ] Unit test: `transition_agent_takes_precedence_over_profile` — `transition_agent = Some("custom")` with `profile.agent = Some("other")` resolves to `"custom"`
- [ ] Unit test: `effective_agent_defaults_to_claude` — no transition agent, no profile, no workers agent resolves to `"claude"`

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
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:56Z | groomed | in_design | philippepascal |