+++
id = "a1b94ea4"
title = "Add outcome field to TransitionConfig with implicit defaults"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a1b94ea4-add-outcome-field-to-transitionconfig-wi"
created_at = "2026-04-30T20:02:08.987471Z"
updated_at = "2026-04-30T21:08:37.433112Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
+++

## Spec

### Problem

The `TransitionConfig` struct in `apm-core/src/config.rs` has no `outcome` field. Any tooling that needs to know whether a transition represents the worker's success path — mock wrappers, dead-end detection in `apm validate`, future UI colouring — must each re-implement the same inference from `CompletionStrategy` and `StateConfig.terminal`. That logic has a canonical definition in `docs/agent-wrappers.md` § "Transition outcomes," but it lives only in prose, not in code.

Adding `pub outcome: Option<String>` to `TransitionConfig` together with a single `resolve_outcome` helper centralises the inference once. Projects that want a more precise label (e.g. marking a transition to `ammend` as `rejected` rather than the inferred `needs_input`) can set the field explicitly; the helper returns the explicit value when present and falls back to the three-rule inference otherwise.

This ticket's scope is the data model and its rules. The field is deliberately inert in the current binary: mock wrappers that read it are a separate ticket (25c92daa). The value delivered here is (a) a stable, typed schema that downstream consumers can rely on without re-deriving the logic, (b) a `resolve_outcome` helper they can call directly, and (c) an annotated `workflow.toml` that makes the shipped default self-describing.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:08Z | groomed | in_design | philippepascal |