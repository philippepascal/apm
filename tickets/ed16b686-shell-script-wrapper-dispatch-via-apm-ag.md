+++
id = "ed16b686"
title = "Shell-script wrapper dispatch via .apm/agents/<name>/wrapper.sh"
state = "in_design"
priority = 0
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ed16b686-shell-script-wrapper-dispatch-via-apm-ag"
created_at = "2026-05-04T16:48:38.984881Z"
updated_at = "2026-05-04T17:27:11.899447Z"
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
- [ ] `.apm/agents/default/wrapper.sh` exists in this repo and is executable (`chmod +x`); `apm init` does not write it to new projects (see Out of scope)
- [ ] `wrapper.sh` invokes the claude binary with `--print --output-format stream-json --verbose --disable-slash-commands`, reading the system prompt from `$APM_SYSTEM_PROMPT_FILE` and the user message from `$APM_USER_MESSAGE_FILE`
- [ ] `wrapper.sh` passes `--dangerously-skip-permissions` when `APM_SKIP_PERMISSIONS` equals `"1"`
- [ ] `wrapper.sh` passes `--model "$APM_MODEL"` when `APM_MODEL` is non-empty
- [ ] The env-var setup for custom wrapper scripts includes `APM_MODEL` (set from `ctx.model`)
- [ ] The `groomed → in_design`, `ammend → in_design`, and `ready → in_progress` transitions in `.apm/workflow.toml` each include `agent = "default"`
- [ ] When no `transition.agent`, no `profile.agent`, and no `workers.agent` are set, the resolved agent name is `"claude"` (unchanged behaviour)
- [ ] Unit test: `transition_agent_takes_precedence_over_profile` — `transition_agent = Some("custom")` with `profile.agent = Some("other")` resolves to `"custom"`
- [ ] Unit test: `effective_agent_defaults_to_claude` — no transition agent, no profile, no workers agent resolves to `"claude"`

### Out of scope

- Adding `agent` to `StateConfig` — states already carry `instructions` for a different, non-spawn purpose
- Per-transition model overrides in `TransitionConfig` — model is an infrastructure concern that stays in worker profiles and workers config
- Any change to the built-in `claude`, `mock-happy`, `mock-sad`, or `debug` wrapper implementations
- Creating `wrapper.sh` files for agent configs other than `default`
- Updating `apm init` to write `wrapper.sh` into new projects — `apm init` does not create `.apm/agents/default/wrapper.sh`; projects that want to customise the wrapper must copy it manually. A follow-up ticket can add this to `init.rs` if needed.
- Changes to the `manifest.toml` format or the "external" parser contract
- Removing the `worker_profiles` mechanism — it remains as the path for infra-only overrides (covered by dependency 6803b88b's out-of-scope boundary)

### Approach

#### 1. Extend TransitionConfig — apm-core/src/config.rs

Add one field to `TransitionConfig`:

```rust
#[serde(default)]
pub agent: Option<String>,
```

No other struct changes are needed.

#### 2. Update effective_spawn_params — apm-core/src/start.rs

Change the function signature to accept a new first argument:

```rust
fn effective_spawn_params(
    transition_agent: Option<&str>,
    profile: Option<&WorkerProfileConfig>,
    workers: &WorkersConfig,
) -> EffectiveWorkerParams
```

Insert resolution at Level 0, before the existing profile and workers checks:

```rust
let agent = transition_agent
    .map(str::to_owned)
    .or_else(|| profile.and_then(|p| p.agent.clone()))
    .or_else(|| workers.agent.clone())
    .unwrap_or_else(|| "claude".to_owned());
```

Update both call sites (`run()` and `spawn_next_worker()`) to pass `triggering_transition.agent.as_deref()` as the first argument. The triggering transition is already captured at both sites (as `triggering_transition_owned` after the 6803b88b changes); extract `.agent.as_deref()` from it.

#### 3. Expose APM_MODEL to custom wrapper scripts — apm-core/src/wrapper/custom.rs

In the function that populates environment variables for custom wrapper invocations, add:

```rust
cmd.env("APM_MODEL", ctx.model.as_deref().unwrap_or(""));
```

Place it alongside the existing `APM_*` variables. This matches what the built-in claude wrapper already has access to via `ctx.model` when building CLI args.

#### 4. Verify manifest.toml is optional — apm-core/src/wrapper/custom.rs

Read `parse_manifest()`. If the function returns an error when `manifest.toml` is absent, change the not-found case to `Ok(ManifestConfig::default())` instead. A `wrapper.sh` with no manifest must be usable without any extra files.

#### 5. Create .apm/agents/default/wrapper.sh

Add the file at `.apm/agents/default/wrapper.sh` and make it executable:

```sh
#!/bin/sh
# APM default wrapper — invokes the claude binary with standard APM arguments.
# Edit this file to customise agent invocation for this project (binary path,
# extra flags, model pinning, etc.).
set -e

sys=$(cat "$APM_SYSTEM_PROMPT_FILE")
msg=$(cat "$APM_USER_MESSAGE_FILE")

set --
[ -n "$APM_MODEL" ] && set -- "$@" --model "$APM_MODEL"
[ -n "$APM_SKIP_PERMISSIONS" ] && set -- "$@" --dangerously-skip-permissions

exec claude \
  --print \
  --output-format stream-json \
  --verbose \
  --disable-slash-commands \
  "$@" \
  --system-prompt "$sys" \
  "$msg"
```

The script builds args incrementally to avoid empty-string artefacts when optional flags are absent.

#### 6. Update .apm/workflow.toml

On the three `command:start` transitions that 6803b88b updates, add `agent = "default"`:

- `groomed → in_design`
- `ammend → in_design`
- `ready → in_progress`

#### 7. Tests — apm-core/src/start.rs

Add two unit tests:

- `transition_agent_takes_precedence_over_profile`: construct a profile with `agent = Some("other".into())` and call `effective_spawn_params(Some("custom"), Some(&profile), &workers_default)`. Assert the returned `agent` field is `"custom"`.
- `effective_agent_defaults_to_claude`: call `effective_spawn_params(None, None, &workers_default)`. Assert the returned `agent` field is `"claude"`.

Update any existing direct calls to `effective_spawn_params` that pass only two arguments to pass `None` as the new first argument — no behaviour change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:56Z | groomed | in_design | philippepascal |
| 2026-05-04T17:03Z | in_design | specd | claude-0504-1656-9e50 |
| 2026-05-04T17:26Z | specd | ammend | philippepascal |
| 2026-05-04T17:27Z | ammend | in_design | philippepascal |