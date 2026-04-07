+++
id = "eea2c9bc"
title = "Add Config helper methods for terminal states and section lookup"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/eea2c9bc-add-config-helper-methods-for-terminal-s"
created_at = "2026-04-07T22:22:22.370019Z"
updated_at = "2026-04-07T22:45:12.041237Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

The same terminal-state lookup pattern is repeated across at least seven locations in the codebase (`archive.rs`, `clean.rs` ×2, `sync.rs`, `verify.rs`, `apm-core/src/ticket.rs`, and `apm-core/src/review.rs`). Each independently reconstructs a set of terminal state IDs by iterating `config.workflow.states`, filtering on `s.terminal`, and collecting into a `HashSet`. Three of those callers also manually insert a hardcoded `"closed"` string as a fallback, while others do not — an inconsistency that is invisible at each individual call site.

A related duplication exists for section name lookups: `apm/src/cmd/spec.rs` and `apm-core/src/ticket.rs` each repeat case-insensitive `eq_ignore_ascii_case` searches against `config.ticket.sections` to validate or retrieve a `TicketSection`.

Both patterns should be centralised as methods on `impl Config` in `apm-core/src/config.rs`, where the existing `actionable_states_for` helper already establishes the pattern. Callers are then migrated to use the helpers, and the hardcoded `"closed"` insertions are audited and removed if redundant.

### Acceptance criteria

- [ ] `Config::terminal_state_ids(&self) -> HashSet<String>` exists on `impl Config` in `apm-core/src/config.rs` and returns exactly the state IDs where `StateConfig::terminal == true`
- [ ] `Config::find_section(&self, name: &str) -> Option<&TicketSection>` exists on `impl Config` and returns the first `TicketSection` whose `name` matches case-insensitively, or `None`
- [ ] `Config::has_section(&self, name: &str) -> bool` exists on `impl Config` and returns `true` iff a matching section exists (delegates to `find_section`)
- [ ] Every inline terminal-state `filter`/`collect` block in `archive.rs`, `clean.rs`, `sync.rs`, `verify.rs`, `apm-core/src/ticket.rs`, and `apm-core/src/review.rs` is replaced with a call to `config.terminal_state_ids()`
- [ ] Every `eq_ignore_ascii_case` section search in `apm/src/cmd/spec.rs` and `apm-core/src/ticket.rs` is replaced with calls to `config.has_section(name)` or `config.find_section(name)`
- [ ] The hardcoded `.insert("closed".to_string())` calls in `archive.rs` and `clean.rs` are removed (after confirming `"closed"` is present in the standard workflow config)
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] No new `clippy` warnings are introduced by the change (`cargo clippy --workspace`)

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
| 2026-04-07T22:22Z | — | new | philippepascal |
| 2026-04-07T22:43Z | new | groomed | apm |
| 2026-04-07T22:45Z | groomed | in_design | philippepascal |