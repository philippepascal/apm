+++
id = "3efea02e"
title = "apm validate: reject merging-completion transition targeting a terminal state"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3efea02e-apm-validate-reject-merging-completion-t"
created_at = "2026-05-29T01:28:21.747382Z"
updated_at = "2026-05-29T01:48:16.247847Z"
+++

## Spec

### Problem

apm validate (validate_config_no_agents in apm-core/src/validate.rs) already validates transition-level rules in the loop around line 375: target state must exist, Merge/PrOrEpicMerge transitions require on_failure, terminal states must have no outgoing transitions, etc.

ADD A RULE: a transition whose completion strategy is Pr, Merge, or PrOrEpicMerge must NOT target a terminal state. This should be an ERROR (consistent with the existing 'terminal but has outgoing transitions' error), not a warning.

RATIONALE: a merging completion is meant to produce a state the supervisor reviews before closing (the implemented -> closed gate). If a merging transition targets a terminal state directly, it collapses merge-and-close, removing the review gate, and makes sync's merge-completed-state set overlap the terminal set — the exact degenerate config that ticket 7dab64ea's runtime guard defends against. Supervisor decision: merge-straight-to-terminal is treated as a misconfiguration and rejected at config time.

WHERE: add the check inside the existing per-transition loop in validate_config_no_agents, alongside the on_failure check (around line 405-417). The set of terminal state IDs is available via config.terminal_state_ids() (note 'closed' is a built-in terminal state even if absent from [[workflow.states]], so the check must treat transition.to == "closed" as terminal too, mirroring how the existing target-exists check special-cases "closed").

MESSAGE FORMAT: consistent with sibling errors, e.g. 'config: state.{state_id}.transition({to}) — completion {strategy} targets terminal state {to}; merging completions must target a non-terminal (review) state'.

RELATIONSHIP: complementary to ticket 7dab64ea (which makes sync robust at runtime by subtracting terminal from the merge-completed set in all passes). This ticket is the config-time safety net; the two are independent and touch different files (validate.rs vs sync.rs/config.rs).

OUT OF SCOPE: changes to sync.rs (covered by 7dab64ea); apm validate --fix auto-remediation for this rule (no single safe fix — resolving it means either retargeting the transition or making the state non-terminal, both supervisor decisions); validating observation 2 (a merge-completed state that is also a normal mid-flow state) — deliberately not validated, too fuzzy to define crisply.

TESTS: validate_config rejects a workflow with a Merge or PrOrEpicMerge transition targeting a terminal state (including the built-in 'closed'); validate_config accepts the default workflow (in_progress -> implemented, where implemented is non-terminal). Existing validate tests must still pass.

### Acceptance criteria

- [x] `apm validate` reports an error when a `Merge` completion transition targets an explicit terminal state
- [x] `apm validate` reports an error when a `PrOrEpicMerge` completion transition targets an explicit terminal state
- [x] `apm validate` reports an error when a `Pr` completion transition targets an explicit terminal state
- [x] `apm validate` reports an error when any merging completion targets the built-in `closed` state, even when `closed` is absent from `[[workflow.states]]`
- [ ] The error message matches the format: `config: state.<id>.transition(<to>) — completion <strategy> targets terminal state <to>; merging completions must target a non-terminal (review) state`
- [ ] `apm validate` accepts a merging completion transition that targets a non-terminal state
- [ ] `apm validate` passes for the default APM workflow (`in_progress → implemented` with `pr_or_epic_merge`, where `implemented` is non-terminal)
- [ ] All existing `apm validate` tests continue to pass

### Out of scope

- Changes to `sync.rs` — the complementary runtime guard is covered by ticket 7dab64ea
- `apm validate --fix` auto-remediation for this rule: no single safe fix exists (the supervisor must either retarget the transition or un-terminal the target state)
- `Pull` completion strategy — it does not produce a branch that lands via PR/merge review and is exempt from this rule
- `CompletionStrategy::None` transitions — no merge occurs, no review gate is bypassed
- Validating that a merge-completed state is also used as a normal mid-flow target (observation 2 in the design discussion — too fuzzy to define crisply)

### Approach

All changes are in `apm-core/src/validate.rs`.

#### 1. Compute terminal-state set once before the outer loop

After line 329 (`let state_ids: HashSet<&str> = …`), add:

```rust
let terminal_ids = config.terminal_state_ids(); // HashSet<String>; already includes "closed"
```

`terminal_state_ids()` (config.rs:655) collects all `terminal = true` state IDs from `workflow.states` and unconditionally inserts `"closed"`, so no additional "closed" special-casing is needed here.

#### 2. Add the check inside the per-transition loop

After the `on_failure` block (after line 427, still inside `for transition in &state.transitions`), add:

```rust
// Merging completions must not target a terminal state.
if matches!(
    transition.completion,
    CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge
) && terminal_ids.contains(transition.to.as_str()) {
    errors.push(format!(
        "config: state.{}.transition({}) — completion {} targets terminal state {}; \
         merging completions must target a non-terminal (review) state",
        state.id,
        transition.to,
        strategy_name(&transition.completion),
        transition.to
    ));
}
```

`terminal_ids.contains(transition.to.as_str())` works because `HashSet<String>` implements `Contains<str>` via `Borrow`.

#### 3. Add tests at the bottom of the `#[cfg(test)]` module

Five new tests, each using `load_config` and `validate_config` (the same helpers already in the module):

- **`merge_completion_targeting_terminal_rejected`** — `in_progress → done` (merge, on_failure=closed), `done` terminal; asserts error contains `state.in_progress.transition(done)` and `targets terminal state`.
- **`pr_or_epic_merge_targeting_terminal_rejected`** — same shape with `pr_or_epic_merge` and `on_failure`; asserts same error pattern.
- **`pr_completion_targeting_terminal_rejected`** — `in_progress → done` (pr), `done` terminal, `[git_host] provider = "github"` to pass the provider check; asserts terminal-state error.
- **`merge_targeting_built_in_closed_rejected`** — `in_progress → closed` (merge, on_failure = some non-terminal state), `closed` not declared in `[[workflow.states]]`; asserts terminal-state error.
- **`merge_targeting_non_terminal_accepted`** — `in_progress → review` (merge, on_failure=closed), `review` non-terminal with a transition to `closed`; asserts no error containing `targets terminal state`.

No test is needed for the default workflow explicitly: `default_workflow_no_dead_end_warning` already builds the default workflow and would fail if `validate_config` produced new errors. Confirming `implemented` is non-terminal in the default config is sufficient (the ticket asserts this).

The existing tests that use `config_with_merge_transition` (which sets `implemented` as terminal) will accumulate the new error in their error lists, but all assertions are `any(|e| e.contains(…))` or narrow-filtered, so none break.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T01:28Z | — | new | philippepascal |
| 2026-05-29T01:28Z | new | groomed | philippepascal |
| 2026-05-29T01:29Z | groomed | in_design | philippepascal |
| 2026-05-29T01:33Z | in_design | specd | claude |
| 2026-05-29T01:47Z | specd | ready | philippepascal |
| 2026-05-29T01:48Z | ready | in_progress | philippepascal |