+++
id = "28ac0f43"
title = "Add state.worker_profile; dispatch reads it (transition fallback retained)"
state = "ammend"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/28ac0f43-add-state-worker-profile-dispatch-reads-"
created_at = "2026-05-31T02:56:42.034762Z"
updated_at = "2026-05-31T19:35:55.075504Z"
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
/// Owner profile for this state. Format: `"agent/role"` (e.g. `"claude/coder"`).
/// When set, the dispatcher uses this as the preferred profile when spawning a
/// worker that enters this state.
#[serde(default)]
pub worker_profile: Option<String>,
```

No other changes to `StateConfig` in this ticket.

#### 2. `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`

Add `worker_profile = "claude/spec-writer"` to the `in_design` state block, and `worker_profile = "claude/coder"` to the `in_progress` state block in both files.

#### 3. `apm-core/src/start.rs` — Shared resolution helper

Extract a private function used by all four dispatch sites:

```rust
fn resolve_dispatch_profile<'a>(
    dest_state_id: &str,
    triggering_transition: Option<&'a TransitionConfig>,
    config: &'a Config,
) -> (&'a str /* profile */, String /* source label */) {
    // 1. Destination state's worker_profile
    if let Some(wp) = config.workflow.states.iter()
        .find(|s| s.id == dest_state_id)
        .and_then(|s| s.worker_profile.as_deref())
    {
        return (wp, format!("workflow.toml state {dest_state_id}.worker_profile"));
    }
    // 2. Firing transition's worker_profile
    if let Some(wp) = triggering_transition.and_then(|tr| tr.worker_profile.as_deref()) {
        let label = if let Some(tr) = triggering_transition {
            format!("workflow.toml transition {} → {}", dest_state_id, tr.to)
            // Note: label uses from→to which the caller can compute
        } else { "workflow.toml transition".to_string() };
        return (wp, label);
    }
    // 3. workers.default
    if let Some(wp) = config.workers.default.as_deref() {
        return (wp, "workers.default".to_string());
    }
    // 4. Built-in fallback
    ("claude/coder", "built-in fallback".to_string())
}
```

Because the helper needs owned data in some paths, the actual implementation may return a `String` rather than `&str`. The exact signature can be adjusted for the borrow checker; the priority order is what matters.

Apply at each dispatch site:

- **`run()`** (line ~477): `new_state` is the destination. Call `resolve_dispatch_profile(&new_state, triggering_transition, &config)`.
- **`run_next()`** (line ~605): destination is `triggering_transition_owned.as_ref().map(|tr| tr.to.as_str()).unwrap_or("in_progress")`. Call helper.
- **`spawn_next_worker()`** (line ~784): same as `run_next()`.
- **`resolve_for_diagnostic()`** (line ~179): destination is `to_id`. Replace the `(worker_profile_str, profile_source)` tuple construction with the helper. The `profile_source` string now correctly names the state when the profile comes from state-level.

The existing `profile_source` format for the transition case must remain `"workflow.toml transition {from} → {to}"` so diagnostic output is unchanged for legacy configs.

#### 4. `apm-core/src/instructions.rs` — State-based role filter

Replace the per-transition `derive_transition_role` call in `format_live_state_machine` with a per-state lookup:

```rust
fn format_live_state_machine(config: &Config, role: Option<&str>) -> String {
    let mut out = String::new();
    out.push_str("| From | To | Command |\n");
    out.push_str("|------|----|----------|\n");

    for state in &config.workflow.states {
        // Determine the role that owns this state (state-level takes precedence).
        let state_role: Option<&str> = state.worker_profile.as_deref()
            .and_then(|wp| wp.split_once('/').map(|(_, r)| r));

        for transition in &state.transitions {
            if let Some(role_name) = role {
                // Include this transition if the state's role matches,
                // OR (legacy fallback) the transition's worker_profile role matches.
                let owned_by_state = state_role == Some(role_name);
                let owned_by_transition = derive_transition_role(transition) == role_name;
                if !owned_by_state && !owned_by_transition {
                    continue;
                }
            }
            let command = if transition.trigger == "command:start" {
                "apm start <id>".to_string()
            } else {
                format!("apm state <id> {}", transition.to)
            };
            out.push_str(&format!("| {} | {} | {} |\n", state.id, transition.to, command));
        }
    }
    out.push('\n');
    out
}
```

The legacy fallback (`owned_by_transition`) ensures configs without `state.worker_profile` still produce correct output during the migration window. After all state profiles are set and transition profiles are dropped (a later ticket), `derive_transition_role` will have no callers and can be deleted then. Do not delete it in this ticket.

Update the existing test `live_state_machine_filters_by_role` in `instructions.rs`: add `worker_profile = "claude/spec-writer"` to `in_design` and `worker_profile = "claude/coder"` to `in_progress` in the inline TOML. Update assertions:
- Coder: assert `in_progress` (FROM), `implemented` (TO of `in_progress → implemented`), and `ready` (TO of `in_progress → ready`) appear; assert `groomed` does not appear as a FROM row.
- Spec-writer: assert `in_design` (FROM), `specd` (TO) appear; the `groomed` TO assertion can remain because the legacy fallback still emits `groomed → in_design` (that transition has `worker_profile = "claude/spec-writer"`).

Update `imperative_table_format_header` test: add `worker_profile = "claude/coder"` to `in_progress` state so the `coder` role filter finds something.

#### 5. `apm-core/src/validate.rs` — `configured_agent_names`

Add a walk over `state.worker_profile` before the existing transition walk:

```rust
for state in &config.workflow.states {
    if let Some(ref wp) = state.worker_profile {
        if let Some((agent, _)) = wp.split_once('/') {
            names.insert(agent.to_string());
        }
    }
    for transition in &state.transitions {
        if let Some(ref wp) = transition.worker_profile {
            if let Some((agent, _)) = wp.split_once('/') {
                names.insert(agent.to_string());
            }
        }
    }
}
```

No test changes needed for this function; the existing tests pass because state-level agents are now additive.

#### 6. `apm-core/src/config.rs` — `implementation_state_ids`

Add a state-level path before the existing transition-based path:

```rust
pub fn implementation_state_ids(&self) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();

    // State-level: states with worker_profile set and not spec-writer.
    for state in &self.workflow.states {
        if let Some(wp) = state.worker_profile.as_deref() {
            if !wp.ends_with("/spec-writer") {
                ids.insert(state.id.clone());
            }
        }
    }

    // Transition-level: command:start with non-spec-writer profile (legacy fallback),
    // and merge-completion targets (always transition-based).
    for state in &self.workflow.states {
        for t in &state.transitions {
            let is_merge_completion = matches!(
                t.completion,
                CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge
            );
            let is_coder_start = t.trigger == "command:start"
                && t.worker_profile.as_deref().is_none_or(|p| !p.ends_with("/spec-writer"));
            if is_merge_completion || is_coder_start {
                ids.insert(t.to.clone());
            }
        }
    }

    ids
}
```

This is additive: both the state-level and transition-level paths contribute to the set. Existing tests (`implementation_state_ids_command_start_no_profile_treated_as_coder`, etc.) continue to pass unchanged because the transition-level path still runs. The default workflow now also contributes `in_progress` via the state-level path.

Add a new test `implementation_state_ids_state_worker_profile_preferred`: workflow has `in_progress.worker_profile = "claude/coder"` with no `command:start` transition; `implementation_state_ids()` returns a set containing `"in_progress"`.

#### 7. New tests in `apm-core/src/start.rs`

Add to the `resolve_for_diagnostic` test block:

- `resolve_for_diagnostic_state_worker_profile_wins`: workflow where the destination state has `worker_profile = "claude/coder"` and the transition also has `worker_profile = "claude/spec-writer"`; assert `diag.worker_profile_str == "claude/coder"` and `diag.profile_source` contains `"state"`.
- `resolve_for_diagnostic_transition_fallback_when_no_state_profile`: workflow with only `transition.worker_profile`; assert `diag.profile_source` contains `"transition"`.

Add dispatch tests (can be unit tests without a full git repo, using the `resolve_dispatch_profile` function directly if extracted):

- Profile precedence: state > transition > workers.default > built-in.
- Legacy shape (no state profile): transition profile used.

### Open questions


### Amendment requests

- [ ] Fix the transition-label pseudocode in Approach section 3. The format string currently shows dest_state_id followed by tr.to, both of which equal the destination state, producing a nonsensical destination-arrow-destination label. The source state must be threaded into the helper or extracted from the calling context. Specify how — either pass source_state_id as a parameter, or have callers format the label themselves.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:56Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:09Z | groomed | in_design | philippepascal |
| 2026-05-31T07:16Z | in_design | specd | claude |
| 2026-05-31T19:35Z | specd | ammend | philippepascal |