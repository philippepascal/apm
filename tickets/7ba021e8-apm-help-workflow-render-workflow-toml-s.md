+++
id = "7ba021e8"
title = "apm help workflow: render workflow.toml schema from WorkflowConfig struct"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ba021e8-apm-help-workflow-render-workflow-toml-s"
created_at = "2026-04-28T19:28:15.496296Z"
updated_at = "2026-04-28T19:52:55.237005Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_workflow()` function in `apm/src/cmd/help.rs` (introduced as a stub by ticket bc89e0a0) returns a placeholder string and does nothing useful. As a result, `apm help workflow` gives users no actionable information about what fields are valid in `.apm/workflow.toml` (or in the `[workflow]` section of `apm.toml`), their types, defaults, or purpose.

The auto-derive infrastructure from ticket 069c3403 can render any `JsonSchema`-annotated struct as a formatted reference table. The types that govern workflow config — `WorkflowConfig`, `StateConfig`, `TransitionConfig`, `PrioritizationConfig`, `SatisfiesDeps`, `CompletionStrategy` — already have `JsonSchema` derived on them (by 069c3403), but most of their fields carry no Rust doc comments today. Since `schemars` converts `/// doc comments` directly into the `description` column of the rendered table, the output would be almost entirely blank without first adding those comments.

This ticket does two things: (1) adds meaningful doc comments to all fields on the workflow-related config types, drawing on the existing spec in `docs/strategy-and-dependencies.md`; (2) replaces the `render_workflow()` stub with a real implementation that calls `apm_core::help_schema::render_schema::<WorkflowConfig>()`.

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
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:52Z | groomed | in_design | philippepascal |