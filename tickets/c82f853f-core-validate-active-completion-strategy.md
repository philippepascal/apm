+++
id = "c82f853f"
title = "core.validate.active_completion_strategy assumes states called in_progress and implemented. Configs can have any state names and several transitions using a completion startegy, so this is the wrong assumption"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c82f853f-core-validate-active-completion-strategy"
created_at = "2026-08-28T00:47:20.903990Z"
updated_at = "2026-08-28T07:14:19.931160Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:47Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:14Z | groomed | in_design | philippepascal |