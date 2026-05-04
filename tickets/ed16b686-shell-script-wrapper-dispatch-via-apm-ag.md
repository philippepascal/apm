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

Checkboxes; each one independently testable.

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