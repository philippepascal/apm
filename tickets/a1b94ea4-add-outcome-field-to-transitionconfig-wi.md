+++
id = "a1b94ea4"
title = "Add outcome field to TransitionConfig with implicit defaults"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a1b94ea4-add-outcome-field-to-transitionconfig-wi"
created_at = "2026-04-30T20:02:08.987471Z"
updated_at = "2026-05-01T01:17:03.747149Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
+++

## Spec

### Problem

The `TransitionConfig` struct in `apm-core/src/config.rs` has no `outcome` field. Any tooling that needs to know whether a transition represents the worker's success path — mock wrappers, dead-end detection in `apm validate`, future UI colouring — must each re-implement the same inference from `CompletionStrategy` and `StateConfig.terminal`. That logic has a canonical definition in `docs/agent-wrappers.md` § "Transition outcomes," but it lives only in prose, not in code.

Adding `pub outcome: Option<String>` to `TransitionConfig` together with a single `resolve_outcome` helper centralises the inference once. Projects that want a more precise label (e.g. marking a transition to `ammend` as `rejected` rather than the inferred `needs_input`) can set the field explicitly; the helper returns the explicit value when present and falls back to the three-rule inference otherwise.

This ticket's scope is the data model and its rules. The field is deliberately inert in the current binary: mock wrappers that read it are a separate ticket (25c92daa). The value delivered here is (a) a stable, typed schema that downstream consumers can rely on without re-deriving the logic, (b) a `resolve_outcome` helper they can call directly, and (c) an annotated `workflow.toml` that makes the shipped default self-describing.

### Acceptance criteria

- [ ] `TransitionConfig` has a `pub outcome: Option<String>` field with `#[serde(default)]` and a doc comment citing the five recognised values
- [ ] A public `resolve_outcome<'a>(transition: &'a TransitionConfig, target_state: &StateConfig) -> &'a str` function exists in `apm-core`
- [ ] `resolve_outcome` returns the explicit outcome string (as `&str`) when `transition.outcome` is `Some`
- [ ] `resolve_outcome` returns `"success"` when `outcome` is `None` and `transition.completion != CompletionStrategy::None`
- [ ] `resolve_outcome` returns `"cancelled"` when `outcome` is `None`, `completion == None`, and `target_state.terminal == true`
- [ ] `resolve_outcome` returns `"needs_input"` when `outcome` is `None`, `completion == None`, and `target_state.terminal == false`
- [ ] Every `[[workflow.states.transitions]]` block in `apm-core/src/default/workflow.toml` contains an explicit `outcome` field
- [ ] `apm validate` emits a `warning:` line (not an error) when the workflow has no reachable `success` outcome from any agent-actionable state
- [ ] `apm validate` exits 0 (success) when the dead-end warning is the only issue
- [ ] Unit tests in `apm-core/src/config.rs` cover all four `resolve_outcome` branches, each as a separate `#[test]`
- [ ] A test asserts that every transition in the default workflow reports a non-empty outcome string via `resolve_outcome`
- [ ] A validate test covers the dead-end-warning path (workflow with an agent-actionable state but no reachable `success` transition)
- [ ] A validate test asserts the dead-end warning is absent for the default workflow (which has a reachable `success` via `in_progress -> implemented`)

### Out of scope

- Mock wrappers (`mock-happy`, `mock-sad`, `mock-random`) reading the `outcome` field — ticket 25c92daa
- `apm validate --fix` auto-populating `outcome` on project `workflow.toml` files (implicit defaults make migration unnecessary)
- Supervisor UI colouring transitions by outcome
- JSON Schema / schemars export changes (automatic via `#[derive(JsonSchema)]` already on `TransitionConfig`)
- Per-profile dead-end analysis in `apm validate` (this ticket warns at global workflow level only; per-profile is a possible follow-up)
- Rejecting unknown outcome values at parse time (custom strings are accepted; tooling treats them as non-success)

### Approach

Five changes across three files: add the `outcome` field to `TransitionConfig`, add the `resolve_outcome` free function, annotate every default-workflow transition with an explicit outcome, add a dead-end reachability warning to `apm validate`, and add the corresponding tests.

#### 1. `apm-core/src/config.rs` — add field to `TransitionConfig`

Add after the `profile` field (~line 365):

```rust
/// Semantic outcome of this transition from the worker's perspective.
/// Recognised values: `success`, `needs_input`, `blocked`, `rejected`, `cancelled`.
/// Custom values are accepted but treated as non-success by tooling.
/// When omitted, `resolve_outcome` applies implicit defaults; see that function.
#[serde(default)]
pub outcome: Option<String>,
```

#### 2. `apm-core/src/config.rs` — add `resolve_outcome`

Add as a free function at module level (not inside `impl`), below the struct definitions:

```rust
/// Returns the effective outcome label for `transition`.
///
/// Uses the explicit `outcome` field when set; otherwise applies implicit defaults in order:
/// 1. `completion` strategy is set (non-`None`) → `"success"`
/// 2. `target_state.terminal` is true → `"cancelled"`
/// 3. Otherwise → `"needs_input"`
pub fn resolve_outcome<'a>(
    transition: &'a TransitionConfig,
    target_state: &StateConfig,
) -> &'a str {
    if let Some(ref o) = transition.outcome {
        return o.as_str();
    }
    if transition.completion != CompletionStrategy::None {
        return "success";
    }
    if target_state.terminal {
        return "cancelled";
    }
    "needs_input"
}
```

The static string returns (`"success"` etc.) coerce to `&'a str` because `'static: 'a`.

#### 3. `apm-core/src/default/workflow.toml` — annotate every transition

Add `outcome = "<value>"` to each `[[workflow.states.transitions]]` block. The mapping rule matches the implicit defaults exactly, so these annotations are self-documenting, not overrides:

| Condition | `outcome` value |
|---|---|
| transition has `completion` set | `"success"` |
| target state is `closed` (terminal) | `"cancelled"` |
| all other transitions | `"needs_input"` |

No transition in the default workflow uses `rejected` or `blocked` explicitly — those values exist for project-level customisation. Every explicit value set here matches what `resolve_outcome` would infer anyway.

Mapping for each transition (implementer: verify `completion` field in the file before writing; `completion`-carrying transitions are the authoritative source of `"success"`):

- `new → groomed` (no completion, non-terminal) → `"needs_input"`
- `new → closed` (terminal) → `"cancelled"`
- `groomed → in_design` (no completion, non-terminal) → `"needs_input"`
- `groomed → closed` → `"cancelled"`
- `question → groomed` → `"needs_input"`
- `question → closed` → `"cancelled"`
- `specd → ready` → `"needs_input"`
- `specd → ammend` → `"needs_input"`
- `specd → closed` → `"cancelled"`
- `ammend → specd` → `"needs_input"`
- `ammend → question` → `"needs_input"`
- `ammend → in_design` → `"needs_input"`
- `ammend → closed` → `"cancelled"`
- `in_design → specd` — if `completion` is set → `"success"`, else `"needs_input"`
- `in_design → question` → `"needs_input"`
- `in_design → ammend` → `"needs_input"`
- `in_design → closed` → `"cancelled"`
- `ready → in_progress` — if `completion` is set → `"success"`, else `"needs_input"`
- `ready → ammend` → `"needs_input"`
- `ready → specd` → `"needs_input"`
- `ready → closed` → `"cancelled"`
- `in_progress → implemented` — has `completion` (merge or pr_or_epic_merge) → `"success"`
- `in_progress → blocked` → `"needs_input"`
- `in_progress → ready` → `"needs_input"`
- `in_progress → ammend` → `"needs_input"`
- `in_progress → closed` → `"cancelled"`
- `blocked → ready` → `"needs_input"`
- `blocked → closed` → `"cancelled"`
- `implemented → ready` → `"needs_input"`
- `implemented → ammend` → `"needs_input"`
- `implemented → in_progress` → `"needs_input"`
- `implemented → closed` → `"cancelled"`
- `merge_failed → implemented` — check `completion`; apply rule
- `merge_failed → in_progress` → `"needs_input"`

#### 4. `apm-core/src/validate.rs` — dead-end warning in `validate_warnings`

Extend `validate_warnings(config: &Config) -> Vec<String>` with a reachability check after the existing docker check:

```
1. Build HashMap<&str, &StateConfig> indexed by state.id for O(1) target lookup.
2. Collect agent-startable state IDs: states where actionable contains "agent" or "any".
   If no such states exist, skip the check — the workflow may be supervisor-only by design.
3. BFS from each startable state ID, tracking visited state IDs to avoid cycles.
   For each visited state, iterate its transitions:
   - Call resolve_outcome(t, lookup[t.to]) for each transition t.
   - If any result == "success": success is reachable — return without warning.
   - Otherwise enqueue t.to if not yet visited.
4. If BFS completes without finding a "success" outcome, push:
   "workflow has no reachable 'success' outcome from any agent-actionable state; \
    workers may never complete successfully"
```

This is O(states × transitions) — negligible for real workflows.

#### 5. Tests

**`apm-core/src/config.rs` `#[cfg(test)]`** — four new unit tests for `resolve_outcome`:

- `resolve_outcome_explicit_override`: `outcome = Some("rejected")`, `completion = None`, non-terminal target → `"rejected"`
- `resolve_outcome_implicit_success`: `outcome = None`, `completion = Merge`, any target → `"success"`
- `resolve_outcome_implicit_cancelled`: `outcome = None`, `completion = None`, `target.terminal = true` → `"cancelled"`
- `resolve_outcome_implicit_needs_input`: `outcome = None`, `completion = None`, `target.terminal = false` → `"needs_input"`

Construct minimal `TransitionConfig` and `StateConfig` values inline; set only the fields each test cares about.

**`apm-core/src/init.rs` or `apm-core/src/config.rs`** — extend `default_workflow_toml_is_valid` or add a sibling test:

Parse the default workflow; build a state-by-id map; for each state's transitions, call `resolve_outcome(t, target)`; assert the result is one of `["success", "needs_input", "blocked", "rejected", "cancelled"]`. This guards against future regressions.

**`apm-core/src/validate.rs` `#[cfg(test)]`** — two new tests:

- `dead_end_workflow_warning_emitted`: construct a minimal `Config` with one `actionable = ["agent"]` state whose only transition leads to a non-terminal, no-completion state with no further exit. Assert `validate_warnings` returns a vec whose first item contains the string `"success"`.
- `default_workflow_no_dead_end_warning`: load the default config (same helper used by existing tests). Assert no item in `validate_warnings` is the dead-end warning string, since `in_progress → implemented` with `completion = merge` is reachable from the agent-actionable `in_progress` state.

### 1. `apm-core/src/config.rs` — add field to `TransitionConfig`

Add after the `profile` field (~line 365):

```rust
/// Semantic outcome of this transition from the worker's perspective.
/// Recognised values: `success`, `needs_input`, `blocked`, `rejected`, `cancelled`.
/// Custom values are accepted but treated as non-success by tooling.
/// When omitted, `resolve_outcome` applies implicit defaults; see that function.
#[serde(default)]
pub outcome: Option<String>,
```

### 2. `apm-core/src/config.rs` — add `resolve_outcome`

Add as a free function at module level (not inside `impl`), below the struct definitions:

```rust
/// Returns the effective outcome label for `transition`.
///
/// Uses the explicit `outcome` field when set; otherwise applies implicit defaults in order:
/// 1. `completion` strategy is set (non-`None`) → `"success"`
/// 2. `target_state.terminal` is true → `"cancelled"`
/// 3. Otherwise → `"needs_input"`
pub fn resolve_outcome<'a>(
    transition: &'a TransitionConfig,
    target_state: &StateConfig,
) -> &'a str {
    if let Some(ref o) = transition.outcome {
        return o.as_str();
    }
    if transition.completion != CompletionStrategy::None {
        return "success";
    }
    if target_state.terminal {
        return "cancelled";
    }
    "needs_input"
}
```

The static string returns (`"success"` etc.) coerce to `&'a str` because `'static: 'a`.

### 3. `apm-core/src/default/workflow.toml` — annotate every transition

Add `outcome = "<value>"` to each `[[workflow.states.transitions]]` block. The mapping rule matches the implicit defaults exactly, so these annotations are self-documenting, not overrides:

| Condition | `outcome` value |
|---|---|
| transition has `completion` set | `"success"` |
| target state is `closed` (terminal) | `"cancelled"` |
| all other transitions | `"needs_input"` |

No transition in the default workflow uses `rejected` or `blocked` explicitly — those values exist for project-level customisation. Every explicit value set here matches what `resolve_outcome` would infer anyway.

Mapping for each transition (implementer: verify `completion` field in the file before writing; `completion`-carrying transitions are the authoritative source of `"success"`):

- `new → groomed` (no completion, non-terminal) → `"needs_input"`
- `new → closed` (terminal) → `"cancelled"`
- `groomed → in_design` (no completion, non-terminal) → `"needs_input"`
- `groomed → closed` → `"cancelled"`
- `question → groomed` → `"needs_input"`
- `question → closed` → `"cancelled"`
- `specd → ready` → `"needs_input"`
- `specd → ammend` → `"needs_input"`
- `specd → closed` → `"cancelled"`
- `ammend → specd` → `"needs_input"`
- `ammend → question` → `"needs_input"`
- `ammend → in_design` → `"needs_input"`
- `ammend → closed` → `"cancelled"`
- `in_design → specd` — if `completion` is set → `"success"`, else `"needs_input"`
- `in_design → question` → `"needs_input"`
- `in_design → ammend` → `"needs_input"`
- `in_design → closed` → `"cancelled"`
- `ready → in_progress` — if `completion` is set → `"success"`, else `"needs_input"`
- `ready → ammend` → `"needs_input"`
- `ready → specd` → `"needs_input"`
- `ready → closed` → `"cancelled"`
- `in_progress → implemented` — has `completion` (merge or pr_or_epic_merge) → `"success"`
- `in_progress → blocked` → `"needs_input"`
- `in_progress → ready` → `"needs_input"`
- `in_progress → ammend` → `"needs_input"`
- `in_progress → closed` → `"cancelled"`
- `blocked → ready` → `"needs_input"`
- `blocked → closed` → `"cancelled"`
- `implemented → ready` → `"needs_input"`
- `implemented → ammend` → `"needs_input"`
- `implemented → in_progress` → `"needs_input"`
- `implemented → closed` → `"cancelled"`
- `merge_failed → implemented` — check `completion`; apply rule
- `merge_failed → in_progress` → `"needs_input"`

### 4. `apm-core/src/validate.rs` — dead-end warning in `validate_warnings`

Extend `validate_warnings(config: &Config) -> Vec<String>` with a reachability check. Insert after the existing docker check:

```
1. Build HashMap<&str, &StateConfig> indexed by state.id for O(1) target lookup.
2. Collect agent-startable state IDs: states where actionable contains "agent" or "any".
3. BFS from each startable state ID, tracking visited state IDs to avoid cycles.
   For each visited state, iterate its transitions:
   - Call resolve_outcome(t, lookup[t.to]) for each transition t.
   - If any result == "success": success is reachable — skip the warning entirely.
   - Otherwise enqueue t.to if not yet visited.
4. If BFS completes without finding a "success" outcome, push:
   "workflow has no reachable 'success' outcome from any agent-actionable state; \
    workers may never complete successfully"
```

This is O(states × transitions) — negligible for real workflows.

Note: skip the check (no warning) if there are no agent-startable states at all, since the workflow may be supervisor-only by design.

### 5. Tests

**`apm-core/src/config.rs` `#[cfg(test)]`** — four new unit tests for `resolve_outcome`:

- `resolve_outcome_explicit_override`: `outcome = Some("rejected")`, `completion = None`, non-terminal target → `"rejected"`
- `resolve_outcome_implicit_success`: `outcome = None`, `completion = Merge`, any target → `"success"`
- `resolve_outcome_implicit_cancelled`: `outcome = None`, `completion = None`, `target.terminal = true` → `"cancelled"`
- `resolve_outcome_implicit_needs_input`: `outcome = None`, `completion = None`, `target.terminal = false` → `"needs_input"`

Construct minimal `TransitionConfig` and `StateConfig` values inline (derive `Default` if needed or set each field explicitly).

**`apm-core/src/init.rs` or `apm-core/src/config.rs`** — extend `default_workflow_toml_is_valid` or add a sibling test:

Parse the default workflow; build a state-by-id map; for each state's transitions, call `resolve_outcome(t, target)`; assert the result is one of `["success", "needs_input", "blocked", "rejected", "cancelled"]`. This guards against future regressions that produce an unexpected outcome string.

**`apm-core/src/validate.rs` `#[cfg(test)]`** — two new tests:

- `dead_end_workflow_warning_emitted`: construct a minimal `Config` with one `actionable = ["agent"]` state whose only transition leads to another non-terminal, no-completion state (forming a cycle with no success exit). Assert `validate_warnings` returns a vec containing a string with `"success"` in the dead-end warning message.
- `default_workflow_no_dead_end_warning`: load the default config (same helper used by existing tests). Assert no item in `validate_warnings` is the dead-end warning string — the default workflow has `in_progress → implemented` with `completion = merge`, which is reachable from `in_progress` (actionable by agents).

### Open questions


### Amendment requests

- [x] The Approach's transition→outcome mapping table only covers ~22 of the 28+ transitions in the current `apm-core/src/default/workflow.toml`. The AC requires every transition to carry an explicit `outcome` field, so the implementer needs the full enumeration. Either (a) expand the mapping table in the Approach to enumerate every transition (preferred — reviewer time spent once is better than implementer time spent guessing), or (b) add an explicit AC step that says "before writing the workflow.toml change, list every transition with its inferred outcome and verify against the Approach's rule set." Pick one. The implicit-default rules will produce the right value either way; this is about doc-completeness so the implementer can verify their work without re-deriving the rules.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:08Z | groomed | in_design | philippepascal |
| 2026-04-30T21:17Z | in_design | specd | claude-0430-2108-e298 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:17Z | ammend | in_design | philippepascal |