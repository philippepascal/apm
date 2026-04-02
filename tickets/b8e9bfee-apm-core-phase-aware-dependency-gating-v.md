+++
id = "b8e9bfee"
title = "apm-core: phase-aware dependency gating via satisfies_deps tags and dep_requires"
state = "specd"
priority = 8
effort = 3
risk = 2
author = "apm"
agent = "38126"
branch = "ticket/b8e9bfee-apm-core-phase-aware-dependency-gating-v"
created_at = "2026-04-02T21:24:08.067343Z"
updated_at = "2026-04-02T22:26:12.567767Z"
+++

## Spec

### Problem

Dependency satisfaction is currently binary: a dep either has `satisfies_deps = true` (implemented/closed) or it doesn't. This means a ticket in `groomed` (waiting to have its spec written) cannot be dispatched until all its deps are fully implemented — even though the only thing needed to write its spec is that the dep's spec exists (`specd`).

In practice this makes the supervisor a bottleneck: every ticket in a dependency chain must be fully implemented before downstream spec work can begin in parallel, eliminating most of the benefit of having a spec-writing phase at all.

The fix is to allow states to declare a named gate tag via `satisfies_deps`, and allow actionable states to declare which gate they require via `dep_requires`. A dep is satisfied for ticket A if the dep's state carries a tag that matches A's required gate (or has `satisfies_deps = true`, or is terminal).

Example for this project's `apm.toml`:
- `specd` gets `satisfies_deps = "spec"` — a dep at this state unblocks downstream spec-writing
- `groomed` gets `dep_requires = "spec"` — only needs deps to reach spec level
- `ready` needs no change — defaults to requiring `satisfies_deps = true` (current behaviour)

### Acceptance criteria

- [ ] `StateConfig` deserialises `satisfies_deps` as either a boolean (`true`/`false`) or a string tag (e.g. `"spec"`) without error
- [ ] `StateConfig` accepts a new optional `dep_requires` string field (e.g. `dep_requires = "spec"`)
- [ ] A ticket whose state has `dep_requires = "X"` is considered unblocked when every dep is in a state that has `satisfies_deps = "X"`, `satisfies_deps = true`, or `terminal = true`
- [ ] A ticket whose state has no `dep_requires` still requires every dep to have `satisfies_deps = true` or `terminal = true` (unchanged behaviour)
- [ ] `apm next` returns a `groomed` ticket (which has `dep_requires = "spec"`) when its only dependency is in state `specd` (which has `satisfies_deps = "spec"`)
- [ ] `apm next` does NOT return a `ready` ticket (no `dep_requires`) when its only dependency is in state `specd` only — it still requires `implemented` or `closed`
- [ ] The project `workflow.toml` has `satisfies_deps = "spec"` on the `specd` state and `dep_requires = "spec"` on the `groomed` state

### Out of scope

- Multiple gate tags per state (e.g. a state satisfying both `"spec"` and `"impl"` gates simultaneously)
- Per-dependency-edge gate overrides (gate is declared on the dependent's state, not on individual dep links)
- Any display or UI changes to how blocked/unblocked status is shown
- Changing how `in_design`, `ammend`, or `question` states interact with dependency gating

### Approach

**Files changed:** `apm-core/src/config.rs`, `apm-core/src/ticket.rs`, `.apm/workflow.toml`

#### 1. `apm-core/src/config.rs` — extend `StateConfig`

Add an untagged enum to handle the union TOML type (bool or string):

```rust
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SatisfiesDeps {
    Bool(bool),
    Tag(String),
}

impl Default for SatisfiesDeps {
    fn default() -> Self { SatisfiesDeps::Bool(false) }
}
```

In `StateConfig`:
- Change `satisfies_deps: bool` to `satisfies_deps: SatisfiesDeps` (keep `#[serde(default)]`)
- Add `#[serde(default)] pub dep_requires: Option<String>`

Make `SatisfiesDeps` `pub` so `ticket.rs` can match on it.

#### 2. `apm-core/src/ticket.rs` — update `dep_satisfied` and `pick_next`

`dep_satisfied` gains a `required_gate: Option<&str>` parameter:

```rust
pub fn dep_satisfied(dep_state: &str, required_gate: Option<&str>, config: &Config) -> bool {
    config.workflow.states.iter()
        .find(|s| s.id == dep_state)
        .map(|s| {
            if s.terminal { return true; }
            match &s.satisfies_deps {
                SatisfiesDeps::Bool(true) => true,
                SatisfiesDeps::Tag(tag) => required_gate == Some(tag.as_str()),
                SatisfiesDeps::Bool(false) => false,
            }
        })
        .unwrap_or(false)
}
```

In `pick_next`, before the dep loop, resolve the candidate ticket's `dep_requires`:

```rust
let required_gate = config.workflow.states.iter()
    .find(|s| s.id == state)
    .and_then(|s| s.dep_requires.as_deref());

// inside the dep loop:
if !dep_satisfied(&dep.frontmatter.state, required_gate, config) {
    return false;
}
```

There is only one existing call site for `dep_satisfied`; update it to pass the gate.

#### 3. `.apm/workflow.toml` — configure the project

- Add `satisfies_deps = "spec"` to `specd`, `ready`, `in_progress`, and `ammend` states — all states where a ticket has a complete, committed spec
- Add `dep_requires = "spec"` to the `groomed` state
- Leave `implemented` and `closed` as `satisfies_deps = true` — no change

Rationale: a `groomed` ticket A that depends on ticket B must remain unblocked as B advances through `ready`, `in_progress`, and `ammend`. Without tagging those states, A re-blocks the moment B advances past `specd`, which is the opposite of the intended behaviour.

#### 4. Tests

Unit tests in `apm-core/src/ticket.rs` (inline `#[cfg(test)]` module):
- `dep_satisfied` with `Tag("spec")` dep state + `required_gate = Some("spec")` -> true
- `dep_satisfied` with `Tag("spec")` dep state + `required_gate = None` -> false
- `dep_satisfied` with `Bool(true)` dep state + `required_gate = None` -> true (backward compat)

Integration test in the existing integration test file:
- Ticket A in `groomed` (with `dep_requires = "spec"`) depends on ticket B
- When B is in `specd` (with `satisfies_deps = "spec"`): `pick_next` returns A
- When B is in `ready` (with `satisfies_deps = "spec"`): `pick_next` still returns A
- When B is only in a pre-spec state (no `satisfies_deps`): `pick_next` does not return A

#### Order of steps

1. Add `SatisfiesDeps` enum and update `StateConfig` in `config.rs`
2. Update `dep_satisfied` and `pick_next` in `ticket.rs`
3. Update `workflow.toml`
4. Add unit and integration tests
5. Run `cargo test --workspace`

### 1. `apm-core/src/config.rs` — extend `StateConfig`

Add an untagged enum to handle the union TOML type:

```rust
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SatisfiesDeps {
    Bool(bool),
    Tag(String),
}

impl Default for SatisfiesDeps {
    fn default() -> Self { SatisfiesDeps::Bool(false) }
}
```

Change `StateConfig`:
- `satisfies_deps: bool` -> `satisfies_deps: SatisfiesDeps`  (keep `#[serde(default)]`)
- Add `#[serde(default)] pub dep_requires: Option<String>`

The `SatisfiesDeps` enum must be `pub` so `ticket.rs` can match on it.

### 2. `apm-core/src/ticket.rs` — update `dep_satisfied` and `pick_next`

**`dep_satisfied`** — add `required_gate: Option<&str>` parameter:

```rust
pub fn dep_satisfied(dep_state: &str, required_gate: Option<&str>, config: &Config) -> bool {
    config.workflow.states.iter()
        .find(|s| s.id == dep_state)
        .map(|s| {
            if s.terminal { return true; }
            match &s.satisfies_deps {
                SatisfiesDeps::Bool(true) => true,
                SatisfiesDeps::Tag(tag) => required_gate == Some(tag.as_str()),
                SatisfiesDeps::Bool(false) => false,
            }
        })
        .unwrap_or(false)
}
```

**`pick_next`** — before the dep loop, resolve the candidate ticket's `dep_requires`:

```rust
let required_gate = config.workflow.states.iter()
    .find(|s| s.id == state)
    .and_then(|s| s.dep_requires.as_deref());

// inside the dep loop:
if !dep_satisfied(&dep.frontmatter.state, required_gate, config) {
    return false;
}
```

Update every existing call to `dep_satisfied` (only the one in `pick_next`) to pass the gate.

### 3. `.apm/workflow.toml` — configure the project

Add to the `specd` state:
```toml
satisfies_deps = "spec"
```

Add to the `groomed` state:
```toml
dep_requires = "spec"
```

Leave `implemented` and `closed` as `satisfies_deps = true` — no change needed.

### 4. Tests

**Unit tests** in `apm-core/src/ticket.rs` (inline):
- `dep_satisfied` with `Tag("spec")` dep state and `required_gate = Some("spec")` -> true
- `dep_satisfied` with `Tag("spec")` dep state and `required_gate = None` -> false
- `dep_satisfied` with `Bool(true)` dep state and `required_gate = None` -> true (backward compat)

**Integration test** in existing integration test file:
- Build a two-ticket scenario: ticket A in `groomed` (with `dep_requires = "spec"`) depends on ticket B
- When B is in `specd` (with `satisfies_deps = "spec"`): `pick_next` returns A
- When B is only in `ready` (no `satisfies_deps`): `pick_next` does not return A

### Order of steps

1. Extend `StateConfig` in `config.rs` (add enum + `dep_requires` field)
2. Update `dep_satisfied` signature and logic in `ticket.rs`
3. Update `pick_next` to resolve and pass `required_gate`
4. Update `workflow.toml`
5. Add unit and integration tests
6. Run `cargo test --workspace`

### Open questions


### Amendment requests

- [ ] Add `satisfies_deps = "spec"` to `ready`, `in_progress`, and `ammend` states in `workflow.toml` — not just `specd`. All three states have a complete spec; without this, a `groomed` ticket A becomes re-blocked the moment its dep B advances past `specd` to `ready` or `in_progress`, which is the opposite of the intended behaviour.
- [ ] Remove the duplicate approach content. The full plan is already written as `####` subsections under `### Approach`; the identical content is repeated as top-level `###` sections below it. Delete the duplicates.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T21:24Z | — | new | apm |
| 2026-04-02T21:25Z | new | groomed | apm |
| 2026-04-02T21:44Z | groomed | in_design | philippepascal |
| 2026-04-02T21:48Z | in_design | specd | claude-0402-2144-spec1 |
| 2026-04-02T22:25Z | specd | ammend | apm |
| 2026-04-02T22:25Z | ammend | in_design | philippepascal |
| 2026-04-02T22:26Z | in_design | specd | apm |