+++
id = 53
title = "Config schema: ticket.sections, state instructions, transition completion and focus_section"
state = "closed"
priority = 5
effort = 3
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-impl-53"
branch = "ticket/0053-config-schema-ticket-sections-state-inst"
created_at = "2026-03-29T19:11:32.157761Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

APM currently hardcodes knowledge about spec sections (Problem, Acceptance criteria, etc.) scattered throughout the Rust code. There is no config-driven way to declare which sections a ticket has, what format they use, or which are required before a transition is allowed. Similarly, the `apm start` delegator has no way to know which subagent instructions to pass when entering a given state, and `apm state` has no way to know whether a transition should open a PR, merge, or do nothing — those behaviours are either hardcoded or absent entirely.

Phase B of the ticket lifecycle design (specified in `initial_specs/TICKET-LIFECYCLE.md`) requires five new config properties to make the delegator, spec writing, and implementation phases work without hardcoding project-specific knowledge in apm itself:

1. `[ticket.sections]` — declares spec sections (name, type, required, placeholder)
2. `instructions` on states — points the delegator to the subagent system prompt
3. `completion` on transitions — controls PR/merge side-effects of `apm state`
4. `focus_section` on transitions — writes a transient frontmatter hint for `apm start --next`
5. `context_section` on transitions — names which section receives `--context` on `apm new`

None of these properties are parsed today. The Rust structs `StateConfig` and `TransitionConfig` in `apm-core/src/config.rs` have no fields for them, and there is no `TicketConfig` or `TicketSection` struct at all. This ticket adds the config schema layer only; no command behaviour changes.

### Acceptance criteria

- [x] `TicketSection` struct exists in `apm-core/src/config.rs` with fields: `name: String`, `type_: SectionType` (deserialized from `"type"`), `required: bool` (default false), `placeholder: Option<String>`
- [x] `SectionType` enum exists with variants `Free`, `Tasks`, `Qa`; deserializes from `"free"`, `"tasks"`, `"qa"`; derives `Debug, Clone, PartialEq, Deserialize`
- [x] `TicketConfig` struct exists with field `sections: Vec<TicketSection>` (default empty vec)
- [x] `Config` struct has field `ticket: TicketConfig` with `#[serde(default)]`
- [x] A `[[ticket.sections]]` block in `apm.toml` with all four fields present parses without error
- [x] A `[[ticket.sections]]` block with only `name` and `type` present (omitting `required` and `placeholder`) parses without error, defaulting `required = false` and `placeholder = None`
- [x] `StateConfig` has field `instructions: Option<String>` (default None); an `[[workflow.states]]` entry with `instructions = "apm.worker.md"` parses correctly
- [x] `TransitionConfig` has field `completion: CompletionStrategy` (default `CompletionStrategy::None`); values `"pr"`, `"merge"`, `"none"` deserialize to the correct variants
- [x] `CompletionStrategy` enum exists with variants `Pr`, `Merge`, `None`; derives `Default` (default = `None`), `Debug, Clone, PartialEq, Deserialize`
- [x] `TransitionConfig` has field `focus_section: Option<String>` (default None); a transition with `focus_section = "Code review"` parses correctly
- [x] `TransitionConfig` has field `context_section: Option<String>` (default None); a transition with `context_section = "Problem"` parses correctly
- [x] All new fields are additive: existing `apm.toml` files that omit all new fields continue to parse without error
- [x] Unit tests in `apm-core/src/config.rs` cover: round-trip parse of a full `[[ticket.sections]]` block; `SectionType` deserialization for all three variants; `CompletionStrategy` deserialization for all three variants and default; `StateConfig` with `instructions`; `TransitionConfig` with `focus_section` and `context_section`
- [x] `cargo test --workspace` passes

### Out of scope

- No changes to any `apm` commands (no `apm new`, `apm start`, `apm state`, `apm spec` changes)
- No use of the new config fields at runtime — fields are parsed and stored only
- No changes to `apm.toml` in the repository (the example TOML is in tests only)
- No precondition evaluation using `[ticket.sections]` (`spec_not_empty` etc. remain as-is)
- No `apm init` template updates
- No documentation files

### Approach

All changes are confined to `apm-core/src/config.rs`.

**New types:**

```rust
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    Free,
    Tasks,
    Qa,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TicketSection {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: SectionType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TicketConfig {
    #[serde(default)]
    pub sections: Vec<TicketSection>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CompletionStrategy {
    Pr,
    Merge,
    #[default]
    None,
}
```

**Modified `Config`** — add one field:
```rust
#[serde(default)]
pub ticket: TicketConfig,
```

**Modified `StateConfig`** — add one field:
```rust
#[serde(default)]
pub instructions: Option<String>,
```

**Modified `TransitionConfig`** — add three fields:
```rust
#[serde(default)]
pub completion: CompletionStrategy,
#[serde(default)]
pub focus_section: Option<String>,
#[serde(default)]
pub context_section: Option<String>,
```

**Example TOML (used in unit tests):**

```toml
[[ticket.sections]]
name        = "Problem"
type        = "free"
required    = true
placeholder = "What is broken or missing?"

[[ticket.sections]]
name     = "Acceptance criteria"
type     = "tasks"
required = true

[[ticket.sections]]
name     = "Open questions"
type     = "qa"
required = false

[[workflow.states]]
id           = "in_progress"
label        = "In Progress"
instructions = "apm.worker.md"

  [[workflow.states.transitions]]
  to              = "implemented"
  trigger         = "manual"
  actor           = "agent"
  completion      = "pr"
  focus_section   = "Code review"
  context_section = "Problem"
```

Unit tests go in a `#[cfg(test)]` block at the bottom of `config.rs`, using inline TOML strings with `toml::from_str`. Each test asserts one scenario (one struct, one variant, one default). No fixture files needed.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:11Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T19:32Z | new | in_design | claude-0329-spec-53 |
| 2026-03-29T19:35Z | in_design | specd | claude-0329-spec-53 |
| 2026-03-29T19:42Z | specd | ready | claude-0329-1200-a1b2 |
| 2026-03-29T19:42Z | ready | in_progress | claude-0329-impl-53 |
| 2026-03-29T19:45Z | in_progress | implemented | claude-0329-impl-53 |
| 2026-03-29T19:47Z | implemented | accepted | claude-0329-1200-a1b2 |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |