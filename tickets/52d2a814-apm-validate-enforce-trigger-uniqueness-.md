+++
id = "52d2a814"
title = "apm validate: enforce trigger-uniqueness and worker_profile shape"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/52d2a814-apm-validate-enforce-trigger-uniqueness-"
created_at = "2026-05-31T02:57:37.160432Z"
updated_at = "2026-05-31T07:26:45.762003Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["071886fc"]
+++

## Spec

### Problem

`apm validate` currently enforces that transition targets exist, that terminal states have no outgoing edges, and that merge completions have `on_failure` set. It does not check three structural properties that, when violated, produce silently broken dispatch behaviour at runtime:

1. **Trigger uniqueness.** A `command:start` transition marks its destination state as a fresh dispatch point. If a second transition (manual or otherwise) can also land on that state, being in the state no longer reliably means the dispatcher should act — the flag becomes ambiguous. No error is emitted today when two transitions converge on the same `command:start` target.

2. **`worker_profile` shape.** Dispatch reads `state.worker_profile` and splits on `/` to extract the agent name and role. A value without a `/`, with empty halves, or with the reserved role `worker` causes a runtime panic or silently falls back to the wrong wrapper. The field is currently accepted without format validation.

3. **`command:start` → dispatch-capable state.** A `command:start` transition that targets a state with no `worker_profile` gives the dispatcher nothing to spawn. This is caught at runtime (no agent is launched) rather than at config-load time.

All three checks are pure additive validation in `validate_config_no_agents`. No existing API changes, no new config fields — malformed `workflow.toml` files are rejected with clear, actionable error messages instead of failing silently at dispatch time.

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
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:26Z | groomed | in_design | philippepascal |