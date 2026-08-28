+++
id = "c82f853f"
title = "core.validate.active_completion_strategy assumes states called in_progress and implemented. Configs can have any state names and several transitions using a completion startegy, so this is the wrong assumption"
state = "in_progress"
priority = 5
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c82f853f-core-validate-active-completion-strategy"
created_at = "2026-08-28T00:47:20.903990Z"
updated_at = "2026-08-28T17:22:58.475595Z"
+++

## Spec

### Problem

`active_completion_strategy` (`apm-core/src/validate.rs:28`) hardcodes its search for
a completion strategy to the transition whose source state id is literally
`"in_progress"` and whose target is literally `"implemented"`:

```rust
config.workflow.states.iter()
    .find(|s| s.id == "in_progress")
    .and_then(|s| s.transitions.iter().find(|t| t.to == "implemented"))
    .map(|t| t.completion.clone())
    .unwrap_or(CompletionStrategy::None)
```

Nothing in the workflow model requires those two literal ids. `apm.toml` lets a
project rename its states to anything (`coding`/`shipped`, `dev`/`merged`,
etc.), and a single workflow can define more than one transition that carries
a `completion` strategy — the built-in default workflow itself has two:
`in_progress → implemented` and `merge_failed → implemented`, both
`pr_or_epic_merge` (`apm-core/src/default/workflow.toml:98-101,161-166`).

When a project renames its states, `active_completion_strategy` silently
returns `CompletionStrategy::None` because it can never find a state literally
called `in_progress`. `check_depends_on_rules` then rejects every
`depends_on` write (`apm set <id> depends_on ...`, `apm new --depends-on ...`)
with "depends_on is not allowed under the none completion strategy" — even
though the project's `.apm/config.toml` has a real `completion = "merge"` (or
`pr`, `pr_or_epic_merge`) configured on its actual in-progress transition. The
error is both wrong and misleading: it blames a strategy the project never
configured. Any project that customises its state names — a documented,
supported customisation — cannot use `depends_on` at all until this is fixed.

Separately, because the function only ever looks at one hardcoded transition,
nothing today checks whether a workflow's other completion-bearing
transitions actually agree with it. If they didn't, `active_completion_strategy`
would pick one arbitrarily and the caller would never know its answer was
only half the story.

### Acceptance criteria

- [ ] `active_completion_strategy` returns the configured strategy for a workflow whose in-progress/implemented-equivalent states use custom ids (e.g. `coding` → `shipped` with `completion = "merge"`), instead of falling back to `none`
- [ ] `active_completion_strategy` resolves correctly when a workflow has multiple transitions that all carry the same non-`none` completion strategy from different source states (e.g. the default workflow's `in_progress → implemented` and `merge_failed → implemented`, both `pr_or_epic_merge`)
- [ ] `active_completion_strategy` still returns `CompletionStrategy::None` when no transition in the workflow sets a non-`none` completion
- [ ] `apm validate` reports a config error when a workflow defines two or more transitions with different non-`none` completion strategies, naming the conflicting states/transitions and the strategies involved
- [ ] `apm set <id> depends_on ...` and `apm new --depends-on ...` correctly enforce the project's actual configured completion-strategy dependency rules on a workflow that uses custom state names, instead of always rejecting with a `none`-strategy error
- [ ] Existing behaviour for the built-in default workflow (`pr_or_epic_merge` on `in_progress → implemented`) is unchanged

### Out of scope

- The similar hardcoded state-name assumptions in `verify_tickets` (`apm-core/src/validate.rs:635-693`), which hardcode `in_progress`/`implemented`/`in_design` for branch and worktree invariant checks. This is a related but distinct bug (different function, different symptoms); file a follow-up ticket if it needs fixing.
- Runtime completion-strategy resolution in `apm state` (`apm-core/src/state.rs:67-93`), which already reads the specific transition being fired rather than a hardcoded pair — it is not affected by this bug and needs no change.
- Any change to the semantics of `check_depends_on_rules` itself (the `Pr`/`Merge`/`Pull`/`PrOrEpicMerge`/`None` dependency rules) — unchanged.
- An automatic fix (`apm validate --fix`) for the new "inconsistent completion strategies" config error — this ticket only adds detection, not auto-repair.
- Renaming `active_completion_strategy` or changing its public signature — it keeps returning `CompletionStrategy` (not `Result`), since the new config-validation rule guarantees at most one distinct non-`none` strategy exists in a valid config.

### Approach

#### Generalize `active_completion_strategy`

File: `apm-core/src/validate.rs`, function at line 28.

Replace the hardcoded `.find(|s| s.id == "in_progress")` /
`.find(|t| t.to == "implemented")` lookup with a scan across every state's
`transitions` (e.g. `config.workflow.states.iter().flat_map(|s| &s.transitions)`)
for the first `TransitionConfig` whose `completion != CompletionStrategy::None`,
regardless of state id or `to` target. Return that transition's
`completion.clone()`, defaulting to `CompletionStrategy::None` when no such
transition exists (unchanged fallback for the no-completion-configured case).

This makes the function agnostic to state names and naturally picks up any
transition carrying a strategy — including a second one like
`merge_failed → implemented` in the default workflow — instead of only ever
looking at one hardcoded pair. Update the function's doc comment: it
currently claims to return "the completion strategy configured for the
`in_progress → implemented` transition"; replace with a description that it
returns the workflow's single configured non-`none` completion strategy
(see the consistency rule below for why "first found" is safe to rely on).

#### Enforce completion-strategy consistency in `apm validate`

File: `apm-core/src/validate.rs`, `validate_config_no_agents` (starts at line
322). This function already has three numbered "Rule N" blocks: trigger/manual
separation (~line 453), `worker_profile` shape (~line 489), and
`command:start` dispatch-target validation (~line 513).

Add a **Rule 4** block after Rule 3 and before the `worktrees.dir` gitignore
check (line 534): walk every state's transitions and collect
`(state.id, transition.to, strategy_name(&transition.completion))` for every
transition whose `completion != CompletionStrategy::None` (`strategy_name` is
the existing private helper at line 36). If more than one distinct strategy
name appears among them, push an error to `errors` naming the conflicting
transitions and strategies, e.g.:

```
config: workflow — inconsistent completion strategies: state.in_progress.transition(implemented)
uses 'merge' but state.hotfix.transition(shipped) uses 'pr'; depends_on validation assumes one
project-wide completion strategy
```

One combined error message listing every offending transition is sufficient;
it does not need to be one error per pair. This is a hard error at the same
severity tier as Rules 1–3, so it surfaces via `apm validate`,
`hash_trip::run` (the config-hash-change gate in `apm/src/hash_trip.rs`), and
`apply_config_migration_fixes`'s internal re-validation
(`apm/src/cmd/validate.rs:174`). With this rule in place, whenever
`validate_config` reports no errors, `active_completion_strategy`'s
first-found-wins behaviour is unambiguous, because at most one distinct
non-`none` strategy can exist in a passing config.

#### Tests

In `apm-core/src/validate.rs`'s inline `mod tests` (the `strategy_config`
helper is around line 1040):

- Add a test using non-default state ids (e.g. `coding` → `shipped` with
  `completion = "merge"`) proving `active_completion_strategy` no longer
  depends on the literal names `in_progress`/`implemented`.
- Add a test with two transitions from different source states that both
  carry the same non-`none` strategy (mirroring `in_progress → implemented`
  + `merge_failed → implemented`), asserting `active_completion_strategy`
  still resolves that single strategy correctly.
- Add a test for the new Rule 4: a config with two transitions carrying
  different non-`none` strategies (one `merge`, one `pr`), asserting
  `validate_config` returns an error that names both transitions.
- Keep the two existing tests (`strategy_finds_in_progress_to_implemented`,
  `strategy_defaults_to_none_when_absent`) passing unchanged.

Run `cargo test --workspace` before submitting; all tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:47Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:14Z | groomed | in_design | philippepascal |
| 2026-08-28T07:19Z | in_design | specd | claude |
| 2026-08-28T17:22Z | specd | ready | philippepascal |
| 2026-08-28T17:22Z | ready | in_progress | philippepascal |
