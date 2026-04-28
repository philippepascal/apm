+++
id = "069c3403"
title = "Auto-derive infrastructure for TOML config schemas"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/069c3403-auto-derive-infrastructure-for-toml-conf"
created_at = "2026-04-28T19:27:37.355186Z"
updated_at = "2026-04-28T19:42:35.829483Z"
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

- [ ] schemars = version 0.8, features derive is present in workspace.dependencies in the root Cargo.toml
- [ ] schemars = workspace true is present in dependencies in apm-core/Cargo.toml
- [ ] apm-core compiles cleanly after adding JsonSchema to all serialized config types in apm-core/src/config.rs
- [ ] Frontmatter in apm-core/src/ticket/ticket_fmt.rs compiles with JsonSchema derive -- the id field custom deserializer does not cause a compilation error (handled via schemars with String)
- [ ] apm_core::help_schema::FieldEntry is a public struct accessible from outside apm-core
- [ ] apm_core::help_schema::schema_entries is callable from outside apm-core for any T: JsonSchema
- [ ] apm_core::help_schema::render_schema is callable from outside apm-core for any T: JsonSchema
- [ ] schema_entries for Config includes an entry for agents.max_concurrent with default == Some("3") and required == false
- [ ] schema_entries for Config includes an entry for project.name with required == true
- [ ] schema_entries for Config includes at least one entry whose toml_path starts with workflow.states[]. (array-of-struct paths use [] notation)
- [ ] schema_entries for Config includes an entry for workflow.states[].transitions[].completion with enum_variants containing all five CompletionStrategy TOML values: pr, merge, pull, pr_or_epic_merge, none
- [ ] render_schema for Config returns a non-empty string that contains the literal text agents.max_concurrent
- [ ] cargo test -p apm-core passes with no regressions

### Out of scope

- render_config(), render_workflow(), render_ticket() implementations -- those are sibling tickets d486d183, 7ba021e8, and 14214305 respectively
- The apm help command dispatcher and topic routing (ticket bc89e0a0)
- apm help commands content (ticket 3665e017)
- ANSI/colour formatting or markdown rendering in output
- Pager integration (less/more)
- Publishing a JSON Schema file as a build artifact
- Deriving JsonSchema on apm-server structs
- LocalConfig and LocalWorkersOverride -- internal override file, not a user-facing apm.toml schema
- Ticket, TicketDocument, ChecklistItem structs in ticket_fmt.rs -- not TOML config schemas; only Frontmatter is in scope

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
| 2026-04-28T19:42Z | groomed | in_design | philippepascal |