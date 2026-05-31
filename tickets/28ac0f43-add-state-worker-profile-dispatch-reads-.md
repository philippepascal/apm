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

- [ ] A `workflow.toml` with `worker_profile = "claude/coder"` on a state parses without error; the field is accessible on `StateConfig.worker_profile`.
- [ ] `apm-core/src/default/workflow.toml` has `worker_profile = "claude/spec-writer"` on `in_design` and `worker_profile = "claude/coder"` on `in_progress`.
- [ ] `.apm/workflow.toml` has the same two additions.
- [ ] Dispatching from a state whose **destination** state carries `state.worker_profile` resolves to that profile — even when `transition.worker_profile` is absent on the firing transition.
- [ ] When both `state.worker_profile` (on the destination state) and `transition.worker_profile` (on the firing transition) are set, the state-level value wins.
- [ ] A workflow with only `transition.worker_profile` (no `state.worker_profile`) still dispatches correctly via the transition fallback.
- [ ] `resolve_for_diagnostic` labels the profile source as `"workflow.toml state <name>.worker_profile"` when the profile came from a state, and `"workflow.toml transition <from> → <to>"` when it came from a transition.
- [ ] `apm instructions --role coder` (with the updated default workflow) emits all transitions out of `in_progress` — not only the `command:start` spawn row.
- [ ] `configure_agent_names` (in `validate.rs`) includes agents referenced in `state.worker_profile` fields, in addition to those in `transition.worker_profile`.
- [ ] `implementation_state_ids` returns `in_progress` for the updated default workflow (derived from `in_progress.worker_profile = "claude/coder"`, not from the spawn transition).
- [ ] `cargo test --workspace` passes with all existing and new tests.

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