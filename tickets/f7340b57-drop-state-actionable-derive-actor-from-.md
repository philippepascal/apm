+++
id = "f7340b57"
title = "Drop state.actionable; derive actor from outgoing triggers"
state = "specd"
priority = 0
effort = 5
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f7340b57-drop-state-actionable-derive-actor-from-"
created_at = "2026-05-31T02:56:19.482471Z"
updated_at = "2026-05-31T07:09:32.430251Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

`StateConfig` carries a `Vec<String>` field `actionable` whose only live values across both the `apm` and `syn` workflows are `"agent"` and `"supervisor"`. The information this field encodes is fully derivable from the outgoing transitions already present on the same state: a state is agent-actionable when it has at least one outgoing transition with `trigger = "command:start"`; otherwise it is supervisor-actionable (provided it is non-terminal). Keeping the explicit field invites future inconsistency, where a manual edit moves a state's transitions without updating `actionable`, silently diverging the two representations.

This ticket removes the field entirely and rewrites every callsite to derive actorhood from the trigger shape. Because `deny_unknown_fields` is added to `StateConfig`, any existing workflow file that still contains `actionable = [...]` will fail to parse with a clear TOML error rather than silently ignoring the stale key. Both `apm-core/src/default/workflow.toml` and `.apm/workflow.toml` are migrated as part of the same change. The result is a smaller config struct, a single source of truth for actor assignment, and a parse-time guard against stale config.

### Acceptance criteria

- [ ] `StateConfig` has no `actionable` field; the struct compiles without it.
- [ ] `StateConfig` is annotated with `deny_unknown_fields`; parsing a `[[workflow.states]]` block that contains `actionable = ["agent"]` returns a TOML error.
- [ ] A workflow TOML with no `actionable` keys parses successfully and all states are accessible.
- [ ] `Config::actionable_states_for("agent")` returns exactly the state IDs that have at least one outgoing transition with `trigger = "command:start"`.
- [ ] `Config::actionable_states_for("supervisor")` returns exactly the non-terminal state IDs that have no `command:start` outgoing transition.
- [ ] `Config::actionable_states_for("engineer")` returns an empty vec.
- [ ] `apm next` returns the same highest-priority ticket before and after the migration when run against the default workflow with tickets in various states.
- [ ] `apm list --actionable agent` returns the same set of tickets before and after the migration.
- [ ] `apm-core/src/default/workflow.toml` contains no `actionable` lines.
- [ ] `.apm/workflow.toml` contains no `actionable` lines.
- [ ] `cargo test --workspace` passes with no failures.

### Out of scope

- `worker_profile` changes (separate ticket).
- Workflow transition restructuring (separate ticket).
- Validate-rule additions beyond updating the existing reachability check.
- Help-text and command-reference list updates.
- `apm-server` UI changes beyond keeping the derived `actionable` field in the workflow graph API response.
- Any workflow other than the two files listed in scope (third-party user workflows are migrated by parse-error guidance, not by this ticket).

### Approach

#### 1. `apm-core/src/config.rs` — struct and method

In `StateConfig`:
- Delete the `actionable` field and its doc comment (currently `pub actionable: Vec<String>`).
- Add `#[serde(deny_unknown_fields)]` to the `StateConfig` derive block. Note: `StateConfig` already derives `Deserialize` and `JsonSchema`; add the serde attribute directly above the `pub struct StateConfig` line.

Rewrite `Config::actionable_states_for`:
```rust
pub fn actionable_states_for(&self, actor: &str) -> Vec<String> {
    match actor {
        "agent" => self.workflow.states.iter()
            .filter(|s| s.transitions.iter().any(|t| t.trigger == "command:start"))
            .map(|s| s.id.clone())
            .collect(),
        "supervisor" => self.workflow.states.iter()
            .filter(|s| !s.terminal
                && !s.transitions.iter().any(|t| t.trigger == "command:start"))
            .map(|s| s.id.clone())
            .collect(),
        _ => vec![],
    }
}
```

Update the unit test `actionable_states_for_agent_includes_ready` (line ~1013): remove `actionable = ["agent"]` and `actionable = ["supervisor"]` from the inline TOML. Add a `command:start` transition on `ready` so it remains agent-actionable. The test assertion stays the same.

#### 2. `apm-core/src/default/workflow.toml`

Delete every `actionable = [...]` line. There are eight of them: on `groomed`, `question`, `specd`, `ammend`, `ready`, `blocked`, `implemented`, and `merge_failed`.

#### 3. `.apm/workflow.toml`

Same deletion: remove all `actionable = [...]` lines. Same set of states.

#### 4. `apm-core/src/validate.rs`

At the dead-end reachability check (line ~682), replace:
```rust
.filter(|s| s.actionable.iter().any(|a| a == "agent" || a == "any"))
```
with:
```rust
.filter(|s| s.transitions.iter().any(|t| t.trigger == "command:start"))
```

The test fixtures in `validate.rs` (lines ~1682, ~1705) that contain `actionable = ["agent"]` in inline TOML strings must also have those lines removed.

#### 5. `apm-server/src/handlers/workflow.rs`

`StateNode` has `pub actionable: Vec<String>`. Keep the field in the API response (removing it would be a breaking API change for any UI consumer). Compute it from transitions instead of copying from the config field:
```rust
actionable: if s.transitions.iter().any(|t| t.trigger == "command:start") {
    vec!["agent".to_string()]
} else if !s.terminal {
    vec!["supervisor".to_string()]
} else {
    vec![]
},
```

#### 6. `apm-server/src/handlers/tickets.rs`

At line ~55, replace:
```rust
.filter(|s| !s.terminal && s.id != "new" && s.actionable.iter().any(|a| a == "supervisor"))
```
with:
```rust
.filter(|s| !s.terminal && s.id != "new"
    && !s.transitions.iter().any(|t| t.trigger == "command:start"))
```

#### 7. `apm-server/src/handlers/epics.rs`

At line ~44, replace:
```rust
.map(|s| s.actionable.iter().any(|a| a == "agent"))
```
with:
```rust
.map(|s| s.transitions.iter().any(|t| t.trigger == "command:start"))
```

#### 8. `apm-server/src/workers.rs`

The filter `s.actionable.is_empty()` identifies states where a worker is actively running (not waiting for human input). After removal of the field, the correct equivalent is: the state is the *destination* of at least one `command:start` transition in the workflow. Replace:
```rust
.filter(|s| !s.terminal && !s.worker_end && s.actionable.is_empty())
```
with:
```rust
.filter(|s| {
    let entered_by_start = config.workflow.states.iter()
        .flat_map(|st| st.transitions.iter())
        .any(|t| t.trigger == "command:start" && t.to == s.id);
    !s.terminal && !s.worker_end && entered_by_start
})
```

#### 9. `apm-core/src/ticket/ticket_util.rs`

- `list_filtered` builds an `actionable_map` from `s.actionable` (line ~478). Replace with a derived lookup: for each state, the actionable actors are determined by whether `command:start` transitions exist. The simplest fix: build the map using the same derivation as `actionable_states_for`.

  Replace the `actionable_map` construction and the `actionable_ok` check with:
  ```rust
  let actionable_ok = actionable_filter.is_none_or(|actor| {
      let state = config.workflow.states.iter().find(|s| s.id == fm.state);
      match (actor, state) {
          ("agent", Some(s)) => s.transitions.iter().any(|t| t.trigger == "command:start"),
          ("supervisor", Some(s)) => !s.terminal
              && !s.transitions.iter().any(|t| t.trigger == "command:start"),
          _ => false,
      }
  });
  ```
  Remove the now-unused `actionable_map` variable.

- `test_config_with_states` helper (line ~841): remove `actionable = [\"agent\"]` from the inline TOML format string. To keep the states agent-actionable for tests that rely on it (none currently pass a non-None `actionable_filter`, so this is safe to drop without adding transitions).

- The `make_ticket_with_owner`-style formatted strings at lines ~845, ~1134, ~1188, ~1194: remove `actionable = ["agent"]` where present.

#### 10. `apm-core/src/epic.rs`

`make_state` helper in the test module: drop the `actionable` parameter and the field assignment from the `StateConfig` literal. Update all call sites within that module to remove the argument.

#### 11. `apm-core/src/wrapper/builtin/mod.rs`

`make_state` test helper at line ~263: remove `actionable: vec![]` from the `StateConfig` literal.

#### 12. `apm-core/src/instructions.rs`

Multiple inline TOML config strings in tests contain `actionable = ["agent"]` and `actionable = ["supervisor"]`. Remove all such lines. The tests check state-machine rendering, not actionable filtering, so the assertions are unaffected.

#### 13. `apm-core/src/start.rs`

Inline TOML fixtures at lines ~2009, ~2025, ~2057, ~2259: remove `actionable = [...]` lines. The `start.rs` logic uses `actionable_states_for` which is already updated via step 1.

#### 14. `apm-core/src/prompt.rs` and `apm-core/src/recovery.rs`

Remove `actionable = [...]` lines from inline TOML test fixtures at lines ~280, ~452 (prompt.rs) and ~135 (recovery.rs).

#### 15. `apm/tests/e2e.rs` and `apm/tests/integration.rs`

All inline TOML workflow strings in these test files contain `actionable = [...]` lines. Remove every occurrence. Where a state relies on being agent-actionable (e.g. `ready` in `next_respects_priority_and_actionable_states`), confirm the state already has a `command:start` outgoing transition — if so, removing `actionable` is sufficient. If not (uncommon), add the transition.

The `apm-server/src/main.rs` test fixtures at lines ~1095, ~2555 follow the same pattern: remove `actionable = [...]` lines.

#### 16. `apm-core/src/config.rs` — `apm-core/tests/ticket_create.rs`

Remove `actionable = ["agent"]` from the inline TOML fixture in that test file.

#### Verification

After all changes: `cargo test --workspace` must pass. The `actionable_states_for` rewrite is the single source of truth used by `apm next`, `apm list --actionable`, and `apm start`; no other behaviour changes.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:56Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:05Z | groomed | in_design | philippepascal |
| 2026-05-31T07:09Z | in_design | specd | claude |
