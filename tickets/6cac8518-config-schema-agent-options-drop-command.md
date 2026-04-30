+++
id = "6cac8518"
title = "Config schema: agent + options (drop command/args/model)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cac8518-config-schema-agent-options-drop-command"
created_at = "2026-04-30T20:02:34.693415Z"
updated_at = "2026-04-30T21:17:25.961937Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

The wrapper dispatcher landed in d3b93b95 always resolves to the `claude` built-in regardless of config — there is no config-driven dispatch yet. Projects cannot choose their agent, pass a model name cleanly through the new path, or extend behaviour without modifying Rust. Meanwhile, `[workers] command/args/model` are still the authoritative fields even though wrappers now own CLI construction.\n\nThis ticket wires the config to the dispatcher. After it lands, `[workers] agent = "claude"` selects the built-in; `[workers.options]` passes arbitrary key-value pairs that are forwarded to the wrapper as `APM_OPT_<KEY>` env vars. Model selection moves to `options.model`. Legacy `command`, `args`, and `model` fields remain parseable for backward compatibility but no longer drive spawn behaviour; a one-time deprecation warning is emitted to stderr when they are present without the new `agent` field.\n\nThe desired state: a project sets `agent = "claude"` (or omits it to accept the default) and `options.model = "sonnet"`, and the dispatcher calls `resolve_builtin("claude").spawn(ctx)` with `ctx.options` populated — identical runtime behaviour to today, but driven by the new architecture.

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
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:17Z | groomed | in_design | philippepascal |