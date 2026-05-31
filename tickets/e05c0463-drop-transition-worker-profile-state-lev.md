+++
id = "e05c0463"
title = "Drop transition.worker_profile (state-level is the only source)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e05c0463-drop-transition-worker-profile-state-lev"
created_at = "2026-05-31T02:57:03.550888Z"
updated_at = "2026-05-31T07:16:34.260985Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["28ac0f43"]
+++

## Spec

### Problem

After ticket 28ac0f43 lands, both `StateConfig` and `TransitionConfig` carry `worker_profile`. The state-level field is the authoritative source; the transition-level field is retained only as a fallback during the migration window. With the default and project workflow.toml files updated to carry `worker_profile` at the state level, the transition-level field is now dead code. Leaving it in place keeps fallback paths alive in the dispatch resolution logic, the agent-name scanner, and the instructions formatter — paths that could mask misconfigured workflows and complicate future changes.

This ticket removes `worker_profile` from `TransitionConfig` entirely, adds `#[serde(deny_unknown_fields)]` to enforce the change at parse time, and strips every fallback path that read from the transition-level field. Any workflow.toml that still carries `worker_profile` under a transition block will fail to parse with a message that names the field and describes the fix.

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
| 2026-05-31T07:16Z | groomed | in_design | philippepascal |