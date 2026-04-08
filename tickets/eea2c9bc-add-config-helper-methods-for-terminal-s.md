+++
id = "eea2c9bc"
title = "Add Config helper methods for terminal states and section lookup"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
branch = "ticket/eea2c9bc-add-config-helper-methods-for-terminal-s"
created_at = "2026-04-07T22:22:22.370019Z"
updated_at = "2026-04-08T00:06:17.438707Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

The same terminal-state lookup pattern is repeated across at least seven locations in the codebase (`archive.rs`, `clean.rs` ×2, `sync.rs`, `verify.rs`, `apm-core/src/ticket.rs`, and `apm-core/src/review.rs`). Each independently reconstructs a set of terminal state IDs by iterating `config.workflow.states`, filtering on `s.terminal`, and collecting into a `HashSet`. Three of those callers also manually insert a hardcoded `"closed"` string as a fallback, while others do not — an inconsistency that is invisible at each individual call site.

A related duplication exists for section name lookups: `apm/src/cmd/spec.rs` and `apm-core/src/ticket.rs` each repeat case-insensitive `eq_ignore_ascii_case` searches against `config.ticket.sections` to validate or retrieve a `TicketSection`.

Both patterns should be centralised as methods on `impl Config` in `apm-core/src/config.rs`, where the existing `actionable_states_for` helper already establishes the pattern. Callers are then migrated to use the helpers, and the hardcoded `"closed"` insertions are audited and removed if redundant.

### Acceptance criteria

- [x] `Config::terminal_state_ids(&self) -> HashSet<String>` exists on `impl Config` in `apm-core/src/config.rs` and returns exactly the state IDs where `StateConfig::terminal == true`
- [x] `Config::find_section(&self, name: &str) -> Option<&TicketSection>` exists on `impl Config` and returns the first `TicketSection` whose `name` matches case-insensitively, or `None`
- [x] `Config::has_section(&self, name: &str) -> bool` exists on `impl Config` and returns `true` iff a matching section exists (delegates to `find_section`)
- [x] Every inline terminal-state `filter`/`collect` block in `archive.rs`, `clean.rs`, `sync.rs`, `verify.rs`, `apm-core/src/ticket.rs`, and `apm-core/src/review.rs` is replaced with a call to `config.terminal_state_ids()`
- [x] Every `eq_ignore_ascii_case` section search in `apm/src/cmd/spec.rs` and `apm-core/src/ticket.rs` is replaced with calls to `config.has_section(name)` or `config.find_section(name)`
- [ ] The hardcoded `.insert("closed".to_string())` calls in `archive.rs` and `clean.rs` are removed (after confirming `"closed"` is present in the standard workflow config)
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] No new `clippy` warnings are introduced by the change (`cargo clippy --workspace`)

### Out of scope

- The `terminal || worker_end` combined filter in `apm/src/cmd/workers.rs` and `apm-server/src/workers.rs` — this is a distinct semantic (worker lifecycle boundary) and belongs in a separate ticket if centralisation is desired
- Adding helper methods for any other `StateConfig` or `TicketSection` properties not already repeated at multiple call sites
- Changing the `StateConfig` or `TicketSection` data model
- Updating the standard `.apm.toml` config to ensure `"closed"` is present (assumed already true; if not, that is a separate defect)

### Approach

Add three methods to the existing `impl Config` block in `apm-core/src/config.rs` (after `actionable_states_for`), then migrate all duplicated call sites to use them.

#### New methods on `impl Config`

```rust
/// Returns the IDs of all terminal workflow states (`StateConfig::terminal == true`).
pub fn terminal_state_ids(&self) -> std::collections::HashSet<String> {
    self.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.clone())
        .collect()
}

/// Case-insensitive lookup of a ticket section by name.
pub fn find_section(&self, name: &str) -> Option<&TicketSection> {
    self.ticket.sections.iter()
        .find(|s| s.name.eq_ignore_ascii_case(name))
}

/// Returns `true` if a ticket section with the given name exists (case-insensitive).
pub fn has_section(&self, name: &str) -> bool {
    self.find_section(name).is_some()
}
```

No new imports are needed; `TicketSection` is defined in the same file.

#### Migrate terminal-state call sites

Remove the inline `filter`/`collect` block in each file below and replace with `config.terminal_state_ids()`. Use `HashSet<String>` as the type (no lifetime needed). Remove dead `use std::collections::HashSet` imports if they become unused.

- `apm-core/src/archive.rs` lines 17-26 — remove `.insert("closed".to_string())`
- `apm-core/src/clean.rs` lines 108-115 — remove `.insert("closed"...)`
- `apm-core/src/clean.rs` lines 338-347 — remove `.insert("closed"...)`
- `apm-core/src/sync.rs` lines 20-23
- `apm-core/src/verify.rs` lines 13-16
- `apm-core/src/ticket.rs` lines 743-746
- `apm-core/src/review.rs` lines 42-45

Callers that previously used `HashSet<&str>` should switch to `HashSet<String>`; downstream `.contains(s.id.as_str())` calls remain valid because `HashSet<String>` implements `contains<str>` via `Borrow<str>`.

Before removing the `"closed"` inserts, verify that the standard configs in `testdata/` include a state with `id = "closed"` and `terminal = true`. If any config lacks it, keep the insert at that call site and add a TODO comment.

#### Migrate section-lookup call sites

- `apm/src/cmd/spec.rs` line 47: `config.ticket.sections.iter().any(|s| s.name.eq_ignore_ascii_case(name))` → `config.has_section(name)`
- `apm/src/cmd/spec.rs` line 59: `.iter().find(|s| s.name.eq_ignore_ascii_case(name)).unwrap()` → `config.find_section(name).unwrap()`
- `apm-core/src/ticket.rs` line 510: `.iter().any(|s| s.name.eq_ignore_ascii_case(&section))` → `config.has_section(&section)`
- `apm-core/src/ticket.rs` line 529: `.find(|s| s.name.eq_ignore_ascii_case(name))` → `config.find_section(name)`

#### Verify

Run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` to confirm nothing regressed.

### 1. Add helper methods to `impl Config` — `apm-core/src/config.rs`

Insert three new methods into the existing `impl Config` block (after `actionable_states_for`):

```rust
/// Returns the IDs of all terminal workflow states (where `StateConfig::terminal == true`).
pub fn terminal_state_ids(&self) -> std::collections::HashSet<String> {
    self.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.clone())
        .collect()
}

/// Case-insensitive lookup of a ticket section by name.
pub fn find_section(&self, name: &str) -> Option<&TicketSection> {
    self.ticket.sections.iter()
        .find(|s| s.name.eq_ignore_ascii_case(name))
}

/// Returns `true` if a ticket section with the given name exists (case-insensitive).
pub fn has_section(&self, name: &str) -> bool {
    self.find_section(name).is_some()
}
```

No new imports are needed — `HashSet` is already in scope or can be fully qualified; `TicketSection` is defined in the same file.

### 2. Migrate terminal-state call sites

For each of the files below, remove the inline `filter`/`collect` block and replace with `config.terminal_state_ids()`. Adjust the type annotation to `HashSet<String>` (no lifetime needed). Unused `HashSet` imports from `std::collections` can be removed if they become dead.

- `apm-core/src/archive.rs` lines 17-26 — also remove the `.insert("closed".to_string())` line after confirming redundancy
- `apm-core/src/clean.rs` lines 108-115 — remove `.insert("closed"...)`
- `apm-core/src/clean.rs` lines 338-347 — remove `.insert("closed"...)`
- `apm-core/src/sync.rs` lines 20-23
- `apm-core/src/verify.rs` lines 13-16
- `apm-core/src/ticket.rs` lines 743-746
- `apm-core/src/review.rs` lines 42-45

For callers that previously used `HashSet<&str>` (borrowed refs), switch to `HashSet<String>` and update any downstream `.contains(s.id.as_str())` calls to `.contains(&s.id)` or `.contains(s.id.as_str())` via `HashSet::contains` on `String`/`&str` — this works because `HashSet<String>` implements `contains<Q>` where `Q: Hash + Eq` and `String: Borrow<str>`.

**Confirming "closed" redundancy:** Before removing the inserts, grep the default `testdata/` configs and any `.apm.toml` in the repo for a state with `id = "closed"` and `terminal = true`. If it is always present in config, the inserts are redundant. If it is absent from any config, keep the insert in that call site and add a TODO comment for a future config-hygiene ticket.

### 3. Migrate section-lookup call sites

- `apm/src/cmd/spec.rs` line 47: replace `config.ticket.sections.iter().any(|s| s.name.eq_ignore_ascii_case(name))` → `config.has_section(name)`
- `apm/src/cmd/spec.rs` line 59: replace `.iter().find(|s| s.name.eq_ignore_ascii_case(name)).unwrap()` → `config.find_section(name).unwrap()`
- `apm-core/src/ticket.rs` line 510: replace `.iter().any(|s| s.name.eq_ignore_ascii_case(&section))` → `config.has_section(&section)`
- `apm-core/src/ticket.rs` line 529: replace `.find(|s| s.name.eq_ignore_ascii_case(name))` → `config.find_section(name)`

### 4. Verify

Run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` to confirm nothing regressed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:22Z | — | new | philippepascal |
| 2026-04-07T22:43Z | new | groomed | apm |
| 2026-04-07T22:45Z | groomed | in_design | philippepascal |
| 2026-04-07T22:48Z | in_design | specd | claude-0407-2245-8d88 |
| 2026-04-08T00:03Z | specd | ready | philippepascal |
| 2026-04-08T00:06Z | ready | in_progress | philippepascal |