+++
id = "7ba021e8"
title = "apm help workflow: render workflow.toml schema from WorkflowConfig struct"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ba021e8-apm-help-workflow-render-workflow-toml-s"
created_at = "2026-04-28T19:28:15.496296Z"
updated_at = "2026-04-28T19:28:15.496296Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

Replace the `render_workflow()` stub from ticket bc89e0a0 with a real renderer that uses the auto-derive infrastructure from ticket 069c3403 to render the `WorkflowConfig` struct and its nested types from `apm-core/src/config.rs`.

**Structure to render:**
- `[[workflow.states]]` array — each `StateConfig` with fields: `id`, `label`, `actionable`, `terminal`, `satisfies_deps` (enum: bool or string-tag), `dep_requires`, `worker_end`, `instructions`.
- `[[workflow.states.transitions]]` nested array — each `TransitionConfig` with fields: `to`, `trigger`, `completion` (enum: `merge`, `pr`, `pr_or_epic_merge`, `pull`, `none`), `profile`, `context_section`, `focus_section`, `warning`, `label`.
- `[workflow.prioritization]` — `PrioritizationConfig` with `priority_weight`, `effort_weight`, `risk_weight`.

**Output structure:**
- Top-level explanation that `workflow.states` is an array (each element an object) and that the structure is config-driven (users define their own states and transitions).
- Per field: name, type, default, description from doc comments.
- For enum fields like `completion` and `satisfies_deps`: list the variants and describe each briefly (the spec at `docs/strategy-and-dependencies.md` already describes `completion` strategies — pull from there or reference it).

**Implementation pointers:**
- In `apm/src/cmd/help.rs`: replace the stub for `workflow` topic. Call into `apm_core::help_schema` for the `WorkflowConfig` type.
- Doc comments on `WorkflowConfig`, `StateConfig`, `TransitionConfig`, `PrioritizationConfig`, `SatisfiesDeps`, `CompletionStrategy` may need to be added or improved.

**Out of scope:**
- A full tutorial on workflow design (this is reference, not guide).
- Validation rules (those belong to `apm validate`).
- Examples beyond struct doc comments.

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
| 2026-04-28T19:28Z | — | new | philippepascal |
