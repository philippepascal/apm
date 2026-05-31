+++
id = "e2781682"
title = "apm-server and apm-ui audit: update API and frontend for schema changes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e2781682-apm-server-and-apm-ui-audit-update-api-a"
created_at = "2026-05-31T02:59:20.324716Z"
updated_at = "2026-05-31T07:55:35.899367Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463", "4d20ba2f"]
+++

## Spec

### Problem

After e05c0463 removes `transition.worker_profile` from `TransitionConfig` (adding `#[serde(deny_unknown_fields)]` to enforce the removal) and 4d20ba2f makes `workers.default` a mandatory non-optional `String`, the inline TOML fixture `MERGE_FAILED_WORKFLOW_CONFIG` in `apm-server/src/main.rs` breaks. It declares `worker_profile = "claude/coder"` on the `ready → in_progress` transition block, which will fail deserialization once `deny_unknown_fields` is active. It also lacks a `[workers]` section entirely, which fails validation after 4d20ba2f makes `workers.default` mandatory. A third inconsistency: 071886fc removes `merge_failed → in_progress` from the default workflow, but the test fixture retains it.

The audit of the remaining surfaces found no other breaking changes. The `apm-server` API response types (`TransitionOption`, `StateNode`, `TransitionEdge` in `handlers/workflow.rs`) have never included transition-level `worker_profile` or `role`. The `apm-server/src/models.rs` response structs are clean. All `apm-ui` TypeScript interfaces, component props, and store types reference only fields that are unaffected by the schema changes. The net result is a single test-fixture update in one file.

### Acceptance criteria

- [ ] `cargo test -p apm-server` passes with the updated fixture
- [ ] `MERGE_FAILED_WORKFLOW_CONFIG` in `apm-server/src/main.rs` contains no `worker_profile` key in any `[[workflow.states.transitions]]` block
- [ ] `MERGE_FAILED_WORKFLOW_CONFIG` contains a `[workers]` section with a non-empty `default` value
- [ ] `MERGE_FAILED_WORKFLOW_CONFIG` contains no `merge_failed → in_progress` transition
- [ ] `get_ticket_recovery_options_populated` test finds a `retry_merge` option pointing to `implemented`
- [ ] `list_tickets_merge_failure_state_ids` test still identifies `merge_failed` in `merge_failure_state_ids` and excludes `in_progress`
- [ ] No field named `worker_profile` appears in any `#[derive(serde::Serialize)]` struct in `apm-server/src/models.rs` or `apm-server/src/handlers/workflow.rs`
- [ ] No TypeScript `interface` or `type` in `apm-ui/src/` contains a field named `worker_profile` or a transition-level `role`

### Out of scope

- Schema struct changes in `apm-core` — owned by e05c0463 (`TransitionConfig.worker_profile` removal, `StateConfig.worker_profile` addition) and 4d20ba2f (`WorkersConfig.default` type change)
- Updating `apm-core/src/recovery.rs` `classify_recovery_options` to look up state-level `worker_profile` instead of transition-level — this is a compile-time consequence of e05c0463 and is fixed there
- Removing `merge_failed → in_progress` from `.apm/workflow.toml` and `apm-core/src/default/workflow.toml` — owned by 071886fc
- Test fixture changes in `apm-core/src/recovery.rs` (`DEFAULT_WF`, `shuffled`, `renamed` have `worker_profile` on transitions) — owned by e05c0463
- Help text and documentation sweep — a5cffb01
- Adding `worker_profile` to the `StateNode` response in `handlers/workflow.rs` — the UI workflow graph does not consume it; this is a new feature, not a fix
- New UI features or visual changes

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:59Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:55Z | groomed | in_design | philippepascal |