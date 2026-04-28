+++
id = "069c3403"
title = "Auto-derive infrastructure for TOML config schemas"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/069c3403-auto-derive-infrastructure-for-toml-conf"
created_at = "2026-04-28T19:27:37.355186Z"
updated_at = "2026-04-28T19:32:51.763389Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
+++

## Spec

### Problem

The `apm help config|workflow|ticket` topics need to render structured help from Rust struct definitions (`Config`, `WorkflowConfig`, `TicketConfig` and their nested types in `apm-core/src/config.rs` and `apm-core/src/ticket/ticket_fmt.rs`). This ticket builds the shared infrastructure for that auto-derivation.

**Required output per field:**
- TOML path (e.g., `agents.max_workers_per_epic` or `workflow.states[].transitions[].completion`).
- Type name (string, integer, bool, list-of-X, enum-with-variants, nested struct, etc.).
- Default value, when one exists (serde defaults, hardcoded fallbacks).
- One-line description sourced from the struct's doc comments (`/// ...`).
- Optional: enum variant list when the field is an enum (e.g., `CompletionStrategy` with `merge`, `pr`, `pr_or_epic_merge`, `pull`, `none`).

**Decision in spec phase — pick one:**
- `schemars` crate (derive `JsonSchema`, traverse the schema, render). Adds a dependency. Well-trodden path. Doc comments become `description` automatically.
- Custom proc-macro derive that walks struct fields and emits a metadata table at compile time. No runtime dep. More bespoke output. More upfront work.
- Pure runtime introspection via `serde_introspect` or hand-rolled visitor. Limited; doc comments are not retained at runtime by serde alone, so descriptions would still need a source.

**User's preference is full auto-derive — no hand-written catalog. Pick the path that best satisfies that.**

**Implementation pointers:**
- New module: `apm-core/src/help_schema.rs`.
- Public API: `pub fn render_struct_schema<T>() -> String` (or trait-based, depending on chosen approach).
- This ticket establishes the infrastructure only — no specific topic uses it yet. T4/T5/T6 in this epic consume it.

**Out of scope:**
- Specific topic content (`config`, `workflow`, `ticket` are separate tickets).
- Markdown formatting beyond plain text.
- Translating TOML to/from JSON Schema as a public artifact.

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
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
