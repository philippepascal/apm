+++
id = "79a03767"
title = "Parameterize transition failure landing in workflow.toml (on_failure)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/79a03767-parameterize-transition-failure-landing-"
created_at = "2026-04-28T23:00:16.821798Z"
updated_at = "2026-04-28T23:04:48.869434Z"
depends_on = ["50649e84"]
+++

## Spec

### Problem

**Problem:** `apm-core/src/state.rs:161-184` hardcodes `t.frontmatter.state = "merge_failed"` when the merge in the `in_progress → implemented` transition fails. This couples the code to a specific state name and bypasses `workflow.toml` entirely. Projects that predate the introduction of `merge_failed` end up with tickets in a state their workflow does not declare — unreachable through `apm state` because no transitions are defined out of it.

This is a wart in the design: the workflow.toml is supposed to be the source of truth for the state machine, but the code writes a state value the workflow may not know about. The fix is to make the failure landing pad **a property of the transition declared in workflow.toml**, not a hardcoded literal in code.

**Schema change — add `on_failure` to `TransitionConfig`:**

```toml
[[workflow.states.transitions]]
to         = "implemented"
trigger    = "manual"
completion = "merge"
on_failure = "merge_failed"   # NEW: optional state to land in if completion fails
```

When the completion strategy fails (currently `merge` and `pr_or_epic_merge` with a target), the code reads `on_failure` from the live transition. If set, it writes that state. If absent, the transition errors out as a hard failure — no automatic state change, the user gets a clear message naming the missing config field.

**The state value referenced by `on_failure` must itself be declared in the same workflow.toml.** `apm validate` enforces both: every transition whose completion can fail (`merge`, `pr_or_epic_merge` with target) must have an `on_failure`, and that on_failure must reference an existing state.

**Implementation pointers:**

- `apm-core/src/config.rs` — `TransitionConfig` struct: add `pub on_failure: Option<String>`.
- `apm-core/src/default/workflow.toml` — the `in_progress → implemented` transition (currently `completion = "merge"`) gets `on_failure = "merge_failed"`. The `merge_failed` state remains declared as today (already in the default template).
- `apm-core/src/state.rs:161-184` — replace the hardcoded `"merge_failed".to_string()` with a read of the live transition's `on_failure`. If `None`, return the merge error as a hard failure (no state mutation, no history line). If `Some(state_name)`, write that state and the history entry as today.
- `apm-core/src/validate.rs` (post-`50649e84`) — add two checks:
  1. Transitions with `completion ∈ {merge, pr_or_epic_merge}` and `on_failure` absent → config error.
  2. Transitions whose `on_failure` references a state not declared in this workflow → config error pointing at the unknown state name.
- `apm validate --fix` — port the missing `on_failure` field from the default template's matching transition (matched by `to` state) into the project's workflow.toml. Idempotent.

**Migration of existing projects:**

A project's existing `in_progress → implemented` transition has no `on_failure` field. The hash-trip on workflow.toml change does not catch this (no edit). `apm validate` will surface it on the next mutating command. `apm validate --fix` ports it from the default template. After the fix, the project's workflow.toml has both the `merge_failed` state and the `on_failure = "merge_failed"` pointer — the user's state machine is whole.

**Acceptance pointers:**

- The `TransitionConfig` struct has `on_failure: Option<String>` and round-trips through TOML.
- A fresh `apm init` produces a workflow.toml whose `in_progress → implemented` transition has `on_failure = "merge_failed"`.
- A pre-existing project (no `on_failure` field) → `apm validate` fails with a clear error naming the transition and the missing field.
- `apm validate --fix` on that project adds the field; re-running validate passes.
- Triggering a real merge failure on a properly-configured project lands the ticket in the configured `on_failure` state, with the history entry naming that state.
- Triggering a merge failure on a project where `on_failure` is absent produces a hard error (the transition does not silently change state); the user is told to run `apm validate --fix`.
- A unit test covers the case where `on_failure` references an unknown state — validate flags it.
- A unit test covers a workflow with the `pr_or_epic_merge` strategy: same rule applies (the merge-to-epic path can fail; `on_failure` is required).

**Out of scope:**

- A general `on_success` field for transitions (this ticket only addresses failure landing).
- Other completion strategies (`pr`, `pull`, `none`) — none of them attempt a merge that can fail in the same way.
- Re-architecting the broader workflow schema beyond the new field.

**Cross-ticket interaction:**

Supersedes the closed ticket `e55fcc73` ("apm validate: enforce code-driven states are declared in workflow.toml"), which was based on a wrong premise — that `merge_failed` is a special "system state". It is not. It is a regular state whose name happens to be referenced by the code, and the right fix is to make the workflow's transition declaration the source of that name.

### Acceptance criteria

- [ ] `TransitionConfig` has a `pub on_failure: Option<String>` field that deserializes from TOML correctly: present value → `Some(...)`, absent field → `None`
- [ ] A fresh `apm init` produces a `workflow.toml` where the `in_progress → implemented` transition includes `on_failure = "merge_failed"`
- [ ] `apm validate` on a project whose `in_progress → implemented` transition has `completion = "merge"` or `completion = "pr_or_epic_merge"` but no `on_failure` field emits a config error that names the source state, the `to` state, and the missing field
- [ ] `apm validate` on a project whose transition has `on_failure` referencing a state not declared in `workflow.toml` emits a config error naming the unknown state value
- [ ] `apm validate --fix` on a project missing `on_failure` adds the field (value ported from the matching default-template transition); a subsequent `apm validate` run exits 0
- [ ] `apm validate --fix` is idempotent: running it twice on the same project produces the same `workflow.toml` and exits 0 both times
- [ ] Triggering a real merge failure on a project with a properly configured `on_failure = "merge_failed"` lands the ticket in `merge_failed`, writes a history entry showing the transition, and the command's output reports `new_state = "merge_failed"`
- [ ] Triggering a merge failure on a project where the transition has no `on_failure` returns a hard error, leaves the ticket in its pre-transition state (no state mutation, no history entry), and the error message instructs the user to run `apm validate --fix`
- [ ] A unit test in `validate.rs` covers: a `completion = "merge"` transition with `on_failure` pointing to an undeclared state → `validate_config()` returns an issue containing that state name
- [ ] A unit test in `validate.rs` covers: a `completion = "pr_or_epic_merge"` transition with no `on_failure` field → `validate_config()` returns an issue (same rule applies as for `merge`)

### Out of scope

- An `on_success` field or any general transition hook mechanism — this ticket only addresses the failure landing pad
- Applying `on_failure` semantics to `completion = "pr"`, `completion = "pull"`, or `completion = "none"` — none of these attempt a merge that can fail in the same recoverable way
- Re-architecting the workflow schema or state machine beyond the single new field
- Migrating tickets already stuck in `merge_failed` due to the old hardcoded code path — they are already in the correct state; only future failures are affected
- Surfacing `on_failure` in `apm show`, `apm list`, or any display commands — it is a config field, not a ticket field

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T23:00Z | — | new | philippepascal |
| 2026-04-28T23:01Z | new | groomed | philippepascal |
| 2026-04-28T23:04Z | groomed | in_design | philippepascal |