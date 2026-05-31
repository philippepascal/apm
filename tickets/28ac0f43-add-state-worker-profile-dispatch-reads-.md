+++
id = "28ac0f43"
title = "Add state.worker_profile; dispatch reads it (transition fallback retained)"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/28ac0f43-add-state-worker-profile-dispatch-reads-"
created_at = "2026-05-31T02:56:42.034762Z"
updated_at = "2026-05-31T21:25:04.841127Z"
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

- Dropping `transition.worker_profile` — retained as fallback; removal is the next ticket.
- Removing the built-in `"claude/coder"` fallback — a later ticket that makes `[workers].default` mandatory.
- Modifying any existing workflow transitions or removing redundant ones.
- Trigger uniqueness validation.
- Help text updates.
- Server and UI surfaces (`apm-server/`, `apm-ui/`) — later ticket.
- Adding `#[serde(deny_unknown_fields)]` to `StateConfig` — handled by dependency f7340b57.

### Approach

#### 1. `apm-core/src/config.rs` — Add field to `StateConfig`

Add after `dep_requires`:

```rust
#[serde(default)]
pub worker_profile: Option<String>,
```

No other changes to `StateConfig` in this ticket.

#### 2. `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`

Add `worker_profile = "claude/spec-writer"` to the `in_design` state block, and `worker_profile = "claude/coder"` to the `in_progress` state block in both files.

#### 3. `apm-core/src/start.rs` — Shared resolution helper

Extract a private function with the following signature and priority order:

```rust
fn resolve_dispatch_profile(
    source_state_id: &str,
    dest_state_id: &str,
    triggering_transition: Option<&TransitionConfig>,
    config: &Config,
) -> (String, String) {
    // 1. Destination state's worker_profile
    if let Some(wp) = config.workflow.states.iter()
        .find(|s| s.id == dest_state_id)
        .and_then(|s| s.worker_profile.as_deref())
    {
        return (
            wp.to_string(),
            format!("workflow.toml state {dest_state_id}.worker_profile"),
        );
    }
    // 2. Firing transition's worker_profile
    if let Some(wp) = triggering_transition.and_then(|tr| tr.worker_profile.as_deref()) {
        return (
            wp.to_string(),
            format!("workflow.toml transition {source_state_id} → {dest_state_id}"),
        );
    }
    // 3. workers.default
    if let Some(wp) = config.workers.default.as_deref() {
        return (wp.to_string(), "workers.default".to_string());
    }
    // 4. Built-in fallback
    ("claude/coder".to_string(), "built-in fallback".to_string())
}
```

Returns `(String, String)` throughout — the built-in `"claude/coder"` literal is converted with `.to_string()`, avoiding `&'static str` / owned `String` lifetime conflicts. No `Cow` needed.

Apply at each dispatch site:

**`run()` (line ~476):** `old_state` (source) and `new_state` (dest) are both available. Replace the four-line chain with:

```rust
let (worker_profile_str, _) = resolve_dispatch_profile(
    &old_state,
    &new_state,
    triggering_transition,
    &config,
);
```

**`run_next()` (line ~601):** `old_state` is the source. `triggering_transition_owned` holds the full `Option<TransitionConfig>`. Replace with:

```rust
let dest = triggering_transition_owned.as_ref()
    .map(|tr| tr.to.as_str())
    .unwrap_or("in_progress");
let (worker_profile_str, _) = resolve_dispatch_profile(
    &old_state,
    dest,
    triggering_transition_owned.as_ref(),
    &config,
);
```

**`spawn_next_worker()` (line ~780):** identical pattern to `run_next()` — `old_state`, `triggering_transition_owned` are already computed the same way.

**`resolve_for_diagnostic()` (line ~134):** The current code extracts only `wp_from_transition: Option<String>` from the transition. To use the helper, capture the full transition instead:

```rust
let (dispatchable, resolved_from_state, from_id, to_id, transition_for_resolution) = {
    // ...same lookup logic...
    if let Some(tr) = current {
        (true, ticket_state.clone(), ticket_state.clone(), tr.to.clone(), Some(tr.clone()))
    } else {
        // fallback scan...
        (false, from.clone(), from, to, Some(tr.clone()))
    }
};
```

Then replace the `(worker_profile_str, profile_source)` tuple at line ~180 with:

```rust
let (worker_profile_str, profile_source) = resolve_dispatch_profile(
    &from_id,
    &to_id,
    transition_for_resolution.as_ref(),
    &config,
);
```

The `profile_source` string for the transition case becomes `"workflow.toml transition {from_id} → {to_id}"`, which matches the existing format and keeps diagnostic output unchanged for legacy configs.

#### 4. `apm-core/src/instructions.rs` — State-based role filter

Replace the per-transition filter in `format_live_state_machine` with a per-state lookup:

```rust
for state in &config.workflow.states {
    let state_role: Option<&str> = state.worker_profile.as_deref()
        .and_then(|wp| wp.split_once('/').map(|(_, r)| r));

    for transition in &state.transitions {
        if let Some(role_name) = role {
            let owned_by_state = state_role == Some(role_name);
            let owned_by_transition = derive_transition_role(transition) == role_name;
            if !owned_by_state && !owned_by_transition {
                continue;
            }
        }
        // emit row...
    }
}
```

The legacy `owned_by_transition` fallback ensures configs without `state.worker_profile` still produce correct output. Do not delete `derive_transition_role` in this ticket.

Update `live_state_machine_filters_by_role` test: add `worker_profile` fields to `in_design` and `in_progress` in the inline TOML; update assertions as described in the original spec.

Update `imperative_table_format_header` test: add `worker_profile = "claude/coder"` to `in_progress` so the coder filter finds it.

#### 5. `apm-core/src/validate.rs` — `configured_agent_names`

Add a walk over `state.worker_profile` before the existing transition walk (additive; existing tests unaffected).

#### 6. `apm-core/src/config.rs` — `implementation_state_ids`

Add a state-level path before the existing transition-based path (additive). Both paths contribute to the set.

Add test `implementation_state_ids_state_worker_profile_preferred`: workflow has `in_progress.worker_profile = "claude/coder"` with no `command:start` transition; assert `in_progress` appears in the result.

#### 7. New tests in `apm-core/src/start.rs`

**`resolve_for_diagnostic` tests** (extend existing block):

- `resolve_for_diagnostic_state_worker_profile_wins`: destination state has `worker_profile = "claude/coder"`, transition has `worker_profile = "claude/spec-writer"`; assert `diag.worker_profile_str == "claude/coder"` and `diag.profile_source` contains `"state"`.
- `resolve_for_diagnostic_transition_fallback_when_no_state_profile`: transition has `worker_profile`, destination state has none; assert `diag.profile_source` contains `"transition"`.

**`resolve_dispatch_profile` unit tests** (new block; no git repo needed):

The helper is a pure function — construct `Config` values inline using TOML deserialization or struct literals. Add:

- `dispatch_profile_state_wins_over_transition`: dest state has `worker_profile = "claude/coder"`, transition has `worker_profile = "claude/spec-writer"`; assert returned profile is `"claude/coder"` and source contains `"state"`.
- `dispatch_profile_transition_fallback`: no state profile, transition has `worker_profile = "claude/spec-writer"`; assert `"claude/spec-writer"` returned.
- `dispatch_profile_workers_default_fallback`: neither state nor transition has a profile, `config.workers.default = Some("claude/custom")`; assert `"claude/custom"` returned.
- `dispatch_profile_builtin_fallback`: nothing set anywhere; assert `"claude/coder"` returned.

These four unit tests on `resolve_dispatch_profile` directly verify the priority order at the logic level. Because all three dispatch functions (`run`, `run_next`, `spawn_next_worker`) are updated to call this helper, the unit tests cover the behavioural dispatch path. Integration-level tests via `resolve_for_diagnostic` (above) additionally verify the full end-to-end path through a real git repo.

### Open questions


### Amendment requests

- [x] Fix the transition-label pseudocode in Approach section 3. The format string currently shows dest_state_id followed by tr.to, both of which equal the destination state, producing a nonsensical destination-arrow-destination label. The source state must be threaded into the helper or extracted from the calling context. Specify how — either pass source_state_id as a parameter, or have callers format the label themselves.
- [x] Specify the return type and lifetime for the new profile resolution helper. The static fallback 'claude/coder' is a &static str while values from config are owned Strings. Decide whether to return String for all paths, use Cow, or have callers handle the label formatting. Current spec is ambiguous about borrow semantics and may not compile as written.
- [x] Add test cases for the both-set scenario where state.worker_profile and transition.worker_profile are both present, asserting state wins, in the actual dispatch functions (run, run_next, spawn_next_worker). Currently only resolve_for_diagnostic has a test for this preference. The dispatch sites are the behavioural ones that matter for real worker spawning.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:56Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:09Z | groomed | in_design | philippepascal |
| 2026-05-31T07:16Z | in_design | specd | claude |
| 2026-05-31T19:35Z | specd | ammend | philippepascal |
| 2026-05-31T20:17Z | ammend | in_design | philippepascal |
| 2026-05-31T20:22Z | in_design | specd | claude |
| 2026-05-31T21:04Z | specd | ready | philippepascal |
| 2026-05-31T21:25Z | ready | in_progress | philippepascal |
