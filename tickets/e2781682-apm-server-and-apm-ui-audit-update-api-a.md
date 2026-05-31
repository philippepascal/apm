+++
id = "e2781682"
title = "apm-server and apm-ui audit: update API and frontend for schema changes"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e2781682-apm-server-and-apm-ui-audit-update-api-a"
created_at = "2026-05-31T02:59:20.324716Z"
updated_at = "2026-05-31T08:03:29.649945Z"
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

#### Audit results

A full read of `apm-server/src/{models,handlers/tickets,handlers/workflow,handlers/maintenance,handlers/mod,agents,work,workers}.rs` and all `apm-ui/src/` TypeScript files confirms only one file needs editing: `apm-server/src/main.rs`.

#### apm-server/src/main.rs — MERGE_FAILED_WORKFLOW_CONFIG

Locate the `const MERGE_FAILED_WORKFLOW_CONFIG: &str = r#"..."#;` block (around line 2523). Make three changes:

1. **Move `worker_profile` from transition to state.** Under the `ready` state, remove `worker_profile = "claude/coder"` from the `[[workflow.states.transitions]]` block. Under the `in_progress` state header, add `worker_profile = "claude/coder"` as a field on the state block itself.

2. **Add `[workers]` section.** Insert `[workers]\ndefault = "claude/coder"\n` before the `[tickets]` section. This satisfies 4d20ba2f's mandatory `workers.default` requirement.

3. **Remove `merge_failed → in_progress` transition.** Delete the `[[workflow.states.transitions]]` block under `merge_failed` that has `to = "in_progress"`. Leave only the `to = "implemented"` block.

The updated constant looks like:
```toml
[project]
name = "test"

[workers]
default = "claude/coder"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "merge"
  on_failure = "merge_failed"

[[workflow.states]]
id    = "implemented"
label = "Implemented"

[[workflow.states]]
id         = "merge_failed"
label      = "Merge failed"
actionable = ["supervisor"]

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"
```

#### Test assertion review

No assertion changes are needed.

`get_ticket_recovery_options_populated` asserts `!opts.is_empty()` and that a `retry_merge` option exists pointing to `implemented`. After removing `merge_failed → in_progress`, only the `retry_merge` option remains, which still satisfies both assertions.

`list_tickets_merge_failure_state_ids` asserts `merge_failed` is in `merge_failure_state_ids` and `in_progress` is not. `is_merge_failure_state("merge_failed")` checks whether any merge-completion transition names `merge_failed` as `on_failure`. The `in_progress → implemented` transition (`completion = "merge"`, `on_failure = "merge_failed"`) still exists, so `merge_failed` is still classified correctly.

#### Verification

Run `cargo test -p apm-server` to confirm all tests pass. No UI build or `vitest` changes are required.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:59Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:55Z | groomed | in_design | philippepascal |
| 2026-05-31T08:03Z | in_design | specd | claude |
