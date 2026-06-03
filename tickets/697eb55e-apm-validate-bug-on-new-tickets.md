+++
id = "697eb55e"
title = "apm validate bug on new tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/697eb55e-apm-validate-bug-on-new-tickets"
created_at = "2026-06-02T21:18:41.660057Z"
updated_at = "2026-06-03T01:24:34.530805Z"
+++

## Spec

### Problem

`apm validate` runs integrity checks on every non-terminal ticket. One check calls `TicketDocument::validate(&config.ticket.sections)`, which iterates over sections marked `required = true` and flags any that are empty or (for `tasks` sections) contain no checklist items. This check fires regardless of the ticket's current state, so tickets in `new` (and similarly `groomed`, `in_design`, `question`) are flagged even though they haven't been through the spec-writing phase yet. The `required` field's own docstring says it applies "before the ticket can transition out of in_design" — i.e., it is a spec-completeness check, not a universal invariant.

Additionally, the error variant `ValidationError::NoAcceptanceCriteria` hardcodes the string "Acceptance criteria" in its `Display` impl. This means the error message does not reflect the actual section name from the config, violating the principle that validation rules should be derived from config.

### Acceptance criteria

- [ ] `apm validate` reports no integrity errors for tickets in `new` state when required sections are empty
- [ ] `apm validate` reports no integrity errors for tickets in `groomed`, `in_design`, or `question` state when required sections are empty
- [ ] `apm validate` does report integrity errors for tickets in `specd` state when required sections are empty
- [ ] `apm validate` does report integrity errors for tickets in `ready` and `in_progress` state when required sections are empty
- [ ] The integrity error message for a `tasks` section with no checklist items uses the section name from config (not the hardcoded string "Acceptance criteria")
- [ ] `TicketSection` in config accepts an optional `validate_from_state` field
- [ ] The default `ticket.toml` sets `validate_from_state = "specd"` for the four required sections (Problem, Acceptance criteria, Out of scope, Approach)
- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Changing validation at state-transition time (`apm state` guards) — this ticket only fixes `apm validate`
- Changing the `## Spec` or `## History` structural checks (those are always enforced)
- Adding `validate_from_state` support to the `apm state in_design → specd` transition guard (a separate concern)
- Changing `required` semantics for projects that do not set `validate_from_state`

### Approach

#### 1. Add `validate_from_state` to `TicketSection` — `apm-core/src/config.rs`

Add an optional `validate_from_state: Option<String>` field to `TicketSection`, after `placeholder`:

```rust
/// When set, required-section checks only apply to tickets whose current state
/// is not in the set of states reachable from the initial workflow state without
/// passing through `validate_from_state`. Absent means always validate.
#[serde(default)]
pub validate_from_state: Option<String>,
```

No schema breakage: the field defaults to `None`, so existing `ticket.toml` files that omit it keep their current (always-validate) behaviour.

#### 2. Fix `ValidationError` — `apm-core/src/ticket/ticket_fmt.rs`

Rename `NoAcceptanceCriteria` to `EmptyTasksSection(String)` (carries the section name). Update `Display`:

```rust
Self::EmptyTasksSection(s) => write!(f, "### {s} has no checklist items"),
```

Update the two call sites in `validate()` to pass `sec.name.clone()`:

```rust
errors.push(ValidationError::EmptyTasksSection(sec.name.clone()));
```

Fix any existing tests that match on the `NoAcceptanceCriteria` variant or its rendered string.

#### 3. Add `pre_validation_states` helper and filter in `verify_tickets` — `apm-core/src/validate.rs`

Add a free function:

```rust
/// Returns the set of state IDs that can be reached from the workflow's
/// initial states (those with no incoming transitions) via forward BFS,
/// WITHOUT entering `barrier_state`. These are the "pre-spec" states for
/// which required-section validation should be skipped.
fn pre_validation_states<'a>(
    barrier_state: &str,
    workflow_states: &'a [crate::config::StateConfig],
) -> HashSet<&'a str>
```

Algorithm:
1. Collect initial states: states with no incoming transition from any other state.
2. BFS forward from each initial state, expanding a state's outgoing transitions but stopping at `barrier_state` (do not add it or expand from it).
3. Return all visited state IDs.

In `verify_tickets`, replace the existing section-validation block:

```rust
if let Ok(doc) = t.document() {
    for err in doc.validate(&config.ticket.sections) {
        issues.push(format!("{prefix}: {err}"));
    }
}
```

With a version that filters sections based on `validate_from_state` and the ticket's current state:

```rust
if let Ok(doc) = t.document() {
    let applicable: Vec<&TicketSection> = config.ticket.sections.iter()
        .filter(|s| {
            match &s.validate_from_state {
                None => true,
                Some(barrier) => {
                    let pre = pre_validation_states(barrier, &config.workflow.states);
                    !pre.contains(fm.state.as_str())
                }
            }
        })
        .collect();
    for err in doc.validate_sections(&applicable) {
        issues.push(format!("{prefix}: {err}"));
    }
}
```

Rename `validate` → `validate_sections` on `TicketDocument` (or add an overload) so it accepts `&[&TicketSection]` rather than `&[TicketSection]`. Alternatively, keep the existing `validate(&[TicketSection])` signature and build a filtered `Vec<TicketSection>` (cloned). Either is fine; the cloned approach avoids a signature change.

Note: `pre_validation_states` is O(states²) at worst but the number of states is small (< 20 for any real project), so no caching needed.

#### 4. Update `ticket.toml` files

In both `apm-core/src/default/ticket.toml` and `.apm/ticket.toml`, add `validate_from_state = "specd"` to the four required sections:

```toml
[[ticket.sections]]
name               = "Problem"
type               = "free"
required           = true
validate_from_state = "specd"
placeholder        = "What is broken or missing, and why it matters."
```

(Same for Acceptance criteria, Out of scope, Approach.)

#### 5. Tests

- Add a unit test in `apm-core/src/validate.rs` (inline `#[cfg(test)]` block) that calls `verify_tickets` with a `new`-state ticket lacking required sections and asserts no integrity errors are returned.
- Add a counterpart test with a `specd`-state ticket lacking required sections and asserts the error IS returned.
- Add a unit test for `pre_validation_states` directly, verifying that `new`, `groomed`, `in_design`, `question` are returned as pre-validation states when `barrier = "specd"` against the default workflow.
- Update any existing tests that reference `ValidationError::NoAcceptanceCriteria` to use `EmptyTasksSection`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T21:18Z | — | new | philippepascal |
| 2026-06-03T01:24Z | new | groomed | philippepascal |
| 2026-06-03T01:24Z | groomed | in_design | philippepascal |