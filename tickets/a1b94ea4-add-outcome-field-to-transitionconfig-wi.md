+++
id = "a1b94ea4"
title = "Add outcome field to TransitionConfig with implicit defaults"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a1b94ea4-add-outcome-field-to-transitionconfig-wi"
created_at = "2026-04-30T20:02:08.987471Z"
updated_at = "2026-04-30T20:02:08.987471Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
+++

## Spec

### Problem

Add an explicit `outcome` field to `TransitionConfig` so mock wrappers and tooling can ask 'is this the success path?' without inferring from `completion` strategy or terminal flags. Independent of the wrapper code; lands as a workflow.toml schema change.

**Reference spec:** `docs/agent-wrappers.md` — section 'Transition outcomes'.

**Scope:**
- Add `pub outcome: Option<String>` to `TransitionConfig` in `apm-core/src/config.rs`, with `#[serde(default)]`.
- Recognised values: `success`, `needs_input`, `blocked`, `rejected`, `cancelled`. Custom values are accepted (treated as non-success by tooling).
- Add a helper `pub fn resolve_outcome(transition: &TransitionConfig, target_state: &StateConfig) -> &str` that returns the explicit value if set, otherwise applies the implicit-default rules:
  1. If `completion` is set (any non-`None` strategy) → `success`
  2. Else if target state has `terminal = true` → `cancelled`
  3. Else → `needs_input`
- Add `outcome` to every transition in `apm-core/src/default/workflow.toml` explicitly. The defaults agree with the inference (so this is documentation only) but make the workflow self-describing for new readers and tooling.
- Extend `apm validate` to warn (not error) if a profile would never reach a `success` outcome from any startable state — a dead-end workflow indicates a config mistake worth surfacing. Conservative: warn, don't fail.

**Out of scope:**
- Mock wrappers using the field (separate ticket; this just adds the field and helper).
- UI surfacing outcome (could be a follow-up; the help schema would auto-pick it up via schemars).
- Hash-trip / validate auto-fix to add `outcome` to existing project workflow.tomls — implicit defaults make this unnecessary.

**Tests:**
- Unit tests for `resolve_outcome` covering each implicit rule and the explicit-override case.
- Update existing default-workflow tests to assert each transition has an outcome (explicit or inferred).
- Validate test for the dead-end warning.

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
