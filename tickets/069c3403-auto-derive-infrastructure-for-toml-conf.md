+++
id = "069c3403"
title = "Auto-derive infrastructure for TOML config schemas"
state = "specd"
priority = 0
effort = 5
risk = 4
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/069c3403-auto-derive-infrastructure-for-toml-conf"
created_at = "2026-04-28T19:27:37.355186Z"
updated_at = "2026-04-28T19:48:50.111514Z"
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

**Chosen approach: `schemars` 0.8.** It converts `/// doc comments` into `description` fields automatically, respects `#[serde(default = "fn")]` by calling the default function and serializing the result, and handles nested structs, Vec-of-struct, enums, and HashMap without manual cataloguing. No proc-macro workspace crate is needed; one new dependency is the only trade-off.

---

**1. Add the dependency**

In root `Cargo.toml` `[workspace.dependencies]` add:

    schemars = { version = "0.8", features = ["derive"] }

In `apm-core/Cargo.toml` `[dependencies]` add:

    schemars = { workspace = true }

---

**2. Derive `JsonSchema` in `apm-core/src/config.rs`**

Add `use schemars::JsonSchema;` and append `JsonSchema` to the `#[derive(...)]` list on every type in `Config`'s serialized tree:

`Config`, `ProjectConfig`, `TicketConfig`, `TicketSection`, `SectionType`, `TicketsConfig`, `WorkflowConfig`, `StateConfig`, `TransitionConfig`, `CompletionStrategy`, `SatisfiesDeps`, `PrioritizationConfig`, `AgentsConfig`, `WorktreesConfig`, `SyncConfig`, `LoggingConfig`, `GitHostConfig`, `WorkersConfig`, `WorkerProfileConfig`, `WorkConfig`, `ServerConfig`, `ContextConfig`

Intentionally exclude `LocalConfig` and `LocalWorkersOverride` (internal override file). The `load_warnings` field already carries `#[serde(skip)]`; schemars respects it.

---

**3. Derive `JsonSchema` in `apm-core/src/ticket/ticket_fmt.rs`**

Add `use schemars::JsonSchema;` and `JsonSchema` to `Frontmatter` only. The `id` field uses a custom `deserialize_with` function that schemars cannot inspect; annotate it with `#[schemars(with = "String")]` to tell schemars to treat it as a plain string.

Leave `Ticket`, `TicketDocument`, `ChecklistItem`, `ValidationError` unchanged.

---

**4. Create `apm-core/src/help_schema.rs`**

Public surface:

    pub struct FieldEntry {
        pub toml_path: String,
        pub type_name: String,
        pub default: Option<String>,
        pub description: Option<String>,
        pub enum_variants: Option<Vec<String>>,
        pub required: bool,
    }

    pub fn schema_entries<T: JsonSchema>() -> Vec<FieldEntry>
    pub fn render_schema<T: JsonSchema>() -> String

**`schema_entries` walker:**

Call `schemars::schema_for::<T>()` to get a `RootSchema { schema, definitions, .. }`. Pass the root `SchemaObject`, the definitions map, an empty path prefix, and the root required-field set to a private recursive helper `walk_object`.

`walk_object` iterates `obj.object.properties` (sorted alphabetically). For each `(field_name, field_schema)`:

1. Resolve any `$ref` by looking up the ref name in `definitions`.
2. Build `toml_path`: `field_name` at root, `prefix.field_name` otherwise.
3. Determine `required` from `obj.object.required`.
4. Classify the resolved schema:
   - **Nested struct** (has named properties, not an array): recurse via `walk_object` with `toml_path` as new prefix; emit no `FieldEntry` for the container itself.
   - **Vec of struct** (`instance_type == Array`, items resolves to an object with properties): recurse with `toml_path + "[]"` as new prefix; emit no `FieldEntry` for the array container. `type_name` label is `list-of-<RefName>` for documentation only.
   - **Vec of scalar** (`instance_type == Array`, items is a scalar): emit one `FieldEntry` with `type_name = "list-of-<scalar>"`.
   - **HashMap** (`additional_properties` set, no named properties): emit one `FieldEntry` with `type_name = "map"`; do not recurse.
   - **Enum** (`enum_values` is Some): emit one `FieldEntry` with `type_name = "string"` and `enum_variants = Some(values as strings)`.
   - **anyOf / oneOf** (untagged enum like `SatisfiesDeps`): emit one `FieldEntry` with `type_name` derived from the variant schemas (e.g., `"bool | string"`), `enum_variants = None`.
   - **Scalar** (`instance_type` is String | Integer | Boolean | Number): map to `"string"`, `"integer"`, `"bool"`, `"number"`.
5. `description` from `schema_obj.metadata.description`.
6. `default` from `schema_obj.metadata.default` (a `serde_json::Value`): for `Value::String(s)` emit `s` without quotes; for numbers/bools call `.to_string()`; for arrays/objects call `serde_json::to_string`.

**`render_schema` renderer:**

Call `schema_entries::<T>()`. Emit one plain-text line per entry:

    <toml_path>  <type>  [default: <val>]  # <description>  (variant1 | variant2 | ...)

Column-align using the widest value per column. Omit `default:` when None; omit `#` clause when description is None; append variants in parentheses when `enum_variants` is Some.

---

**5. Export from `apm-core/src/lib.rs`**

Add `pub mod help_schema;`.

---

**6. Unit tests inside `help_schema.rs`**

Four tests in a `#[cfg(test)]` block:

- `agents_max_concurrent_has_default_3`: finds the entry, asserts `default == Some("3")` and `required == false`
- `project_name_is_required`: asserts `required == true`
- `workflow_states_uses_array_notation`: asserts at least one entry has `toml_path` starting with `"workflow.states[]."`
- `completion_strategy_has_enum_variants`: asserts the `workflow.states[].transitions[].completion` entry has `enum_variants` containing `"none"` and `"pr"`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:42Z | groomed | in_design | philippepascal |
| 2026-04-28T19:48Z | in_design | specd | claude-0428-1942-2dc0 |
