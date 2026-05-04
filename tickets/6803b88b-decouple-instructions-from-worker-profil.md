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
| 2026-05-04T16:50Z | groomed | in_design | philippepascal |