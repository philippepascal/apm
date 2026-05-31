+++
id = "28ac0f43"
title = "Add state.worker_profile; dispatch reads it (transition fallback retained)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/28ac0f43-add-state-worker-profile-dispatch-reads-"
created_at = "2026-05-31T02:56:42.034762Z"
updated_at = "2026-05-31T07:09:46.991637Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["f7340b57"]
+++

## Spec

### Problem

Dispatch resolution in `apm-core/src/start.rs` currently reads the worker profile exclusively from the firing `transition.worker_profile`. This means the profile that determines which agent spawns into `in_progress` is declared on the `ready → in_progress` transition rather than on `in_progress` itself. As the workflow grows, every spawn transition must repeat the profile — and the `instructions.rs` role filter can only show transitions tagged with a matching `worker_profile`, so a coder currently sees only the single `ready → in_progress` spawn row, not the full set of state-exits (`in_progress → implemented`, `in_progress → blocked`, etc.) that describe its actual job.

This ticket adds `state.worker_profile: Option<String>` to `StateConfig` and teaches the four dispatch resolution sites to prefer it over `transition.worker_profile`. It also updates the instructions filter to show all transitions out of a state the role owns, and updates `configured_agent_names` and `implementation_state_ids` to read state-level profiles. The old `transition.worker_profile` is retained as a working fallback throughout; no existing configurations break.

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
| 2026-05-31T02:56Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:09Z | groomed | in_design | philippepascal |