+++
id = "697eb55e"
title = "apm validate bug on new tickets"
state = "implemented"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/697eb55e-apm-validate-bug-on-new-tickets"
created_at = "2026-06-02T21:18:41.660057Z"
updated_at = "2026-06-03T02:38:40.133926Z"
+++

## Spec

### Problem

`apm validate` runs integrity checks on every non-terminal ticket. One check calls `TicketDocument::validate(&config.ticket.sections)`, which iterates over sections marked `required = true` and flags any that are empty or (for `tasks` sections) contain no checklist items. This check fires regardless of the ticket's current state, so tickets in `new` (and similarly `groomed`, `in_design`, `question`) are flagged even though they haven't been through the spec-writing phase yet. The `required` field's own docstring says it applies "before the ticket can transition out of in_design" — i.e., it is a spec-completeness check, not a universal invariant.

Additionally, the error variant `ValidationError::NoAcceptanceCriteria` hardcodes the string "Acceptance criteria" in its `Display` impl. This means the error message does not reflect the actual section name from the config, violating the principle that validation rules should be derived from config.

### Acceptance criteria

- [x] `apm validate` reports no integrity errors for tickets in `new` state when required sections are empty
- [x] `apm validate` reports no integrity errors for tickets in `groomed`, `in_design`, or `question` state when required sections are empty
- [x] `apm validate` does report integrity errors for tickets in `specd` state when required sections are empty
- [x] `apm validate` does report integrity errors for tickets in `ready` and `in_progress` state when required sections are empty
- [x] The integrity error message for a `tasks` section with no checklist items uses the section name from config (not the hardcoded string "Acceptance criteria")
- [x] `TicketSection` in config accepts an optional `validate_from_state` field
- [x] The default `ticket.toml` sets `validate_from_state = "specd"` for the four required sections (Problem, Acceptance criteria, Out of scope, Approach)
- [x] `pre_validation_states` with `barrier = "specd"` against the default workflow returns exactly `{new, groomed, in_design, question}`; `closed` is not in the set
- [x] `cargo test --workspace` passes with no regressions

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
1. Collect initial states: states where `state.terminal == false` AND with no incoming transition from any other state. Terminal states are excluded because they may lack explicit incoming transitions in the workflow config (e.g. `closed` after commit e20488b3 made it implicit), which would otherwise cause the naive rule to treat them as initial states and incorrectly include them in the pre-validation set.
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
- Add a unit test for `pre_validation_states` directly, asserting the returned set is exactly `{new, groomed, in_design, question}` — and that `closed` is not present — when `barrier = "specd"` against the default workflow.
- Update any existing tests that reference `ValidationError::NoAcceptanceCriteria` to use `EmptyTasksSection`.

### Open questions


### Amendment requests

- [x] Fix the initial-states detection in pre_validation_states. The current algorithm step 1 says to collect states with no incoming transition from any other state. In the current workflow (post-e20488b3, which made close implicit), the closed state has no explicit incoming transition in workflow.toml, so the naive rule treats closed as an initial state. BFS from closed adds only closed itself (it has no outgoing), but the result then incorrectly includes closed in the pre-validation set. Tickets in closed silently skip required-section checks. Today this is invisible (closed tickets do not get re-validated anyway), but it is wrong-by-luck and would break if another terminal state is added later (abandoned, wontfix, etc). Required change: in step 1 of pre_validation_states, exclude terminal states from the initial set. The terminal flag is already on StateConfig (introduced by e20488b3). One additional filter: skip any state where state.terminal is true before checking for incoming transitions. Also add an AC: pre_validation_states for the default workflow returns exactly the set new, groomed, in_design, question when barrier is specd — closed must not appear. Add a unit test asserting this exact set.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T21:18Z | — | new | philippepascal |
| 2026-06-03T01:24Z | new | groomed | philippepascal |
| 2026-06-03T01:24Z | groomed | in_design | philippepascal |
| 2026-06-03T01:32Z | in_design | specd | claude |
| 2026-06-03T02:11Z | specd | ammend | philippepascal |
| 2026-06-03T02:14Z | ammend | in_design | philippepascal |
| 2026-06-03T02:19Z | in_design | specd | claude |
| 2026-06-03T02:20Z | specd | ready | philippepascal |
| 2026-06-03T02:20Z | ready | in_progress | philippepascal |
| 2026-06-03T02:38Z | in_progress | implemented | claude |
