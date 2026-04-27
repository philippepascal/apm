+++
id = "e845127e"
title = "Extend apm validate to enforce dependency rules across tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e845127e-extend-apm-validate-to-enforce-dependenc"
created_at = "2026-04-27T20:28:41.454959Z"
updated_at = "2026-04-27T21:16:27.693838Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["a3dc64db"]
+++

## Spec

### Problem

`apm validate` currently checks config correctness, ticket state validity, and branch-field consistency. It does not check whether existing tickets' `depends_on` fields satisfy the rules for the currently configured completion strategy.

The spec at `docs/strategy-and-dependencies.md` (section Dependency rules per strategy) defines when dependencies compose safely: under `pr_or_epic_merge`, all deps must share the ticket's epic; under `merge`, all deps must share the ticket's `target_branch`; under `pr` or `none`, no deps are allowed at all. These rules are enforced at write time by ticket a3dc64db, but tickets created before that enforcement existed -- or tickets whose config changed after creation -- can violate the rules silently.

This ticket extends `apm validate` to walk every non-closed ticket and report each one whose `depends_on` violates the active strategy rule. Ticket a3dc64db provides `active_completion_strategy()` and `check_depends_on_rules()` in `apm-core/src/validate.rs`; this ticket adds a sweep function that calls them over all loaded tickets, keeping the rule logic in a single place shared by both the write-time guards and the full-scan validator.

### Acceptance criteria

- [ ] `apm validate` reports an error for each non-closed ticket whose `depends_on` is non-empty and violates the active completion strategy rule
- [ ] `apm validate` reports no `depends_on` error for tickets with an empty or absent `depends_on`
- [ ] `apm validate` skips tickets in the `closed` state when checking `depends_on`
- [ ] `apm validate` reports a `depends_on` error when a dep ID in `depends_on` is not found in the loaded ticket set
- [ ] `apm validate --json` includes each dependency violation in the errors array with kind = depends_on
- [ ] Human-readable output for dependency violations follows the existing format: error [depends_on] #id: message
- [ ] A ticket with a `depends_on` that satisfies the active strategy (correct epic for `pr_or_epic_merge`, correct `target_branch` for `merge`) produces no `depends_on` error
- [ ] When the strategy is `pr` or `none`, any ticket with a non-empty `depends_on` is flagged
- [ ] `apm validate --config-only` does not run dependency checks (tickets are not loaded in config-only mode)
- [ ] `apm validate` exits with a non-zero exit code when any `depends_on` violation is found
- [ ] A pub fn validate_depends_on(config: &Config, tickets: &[Ticket]) -> Vec<(String, String)> exists in apm-core/src/validate.rs with at least 7 unit tests covering: no deps, closed ticket skipped, pr_or_epic_merge same-epic passes, pr_or_epic_merge cross-epic fails, merge same-target passes, merge different-target fails, pr strategy rejects any dep

### Out of scope

- Implementing active_completion_strategy() and check_depends_on_rules() -- those are ticket a3dc64db
- Hash-trip re-validation triggered by config or workflow changes -- ticket b10d957a
- Auto-fix (--fix) for dependency violations -- no safe automatic correction exists
- Enforcing dependency rules at write time (apm new, apm set) -- ticket a3dc64db
- Changing the default completion strategy to pr_or_epic_merge -- ticket 941e57fa
- Epic quiescence checks in apm epic close or apm refresh-epic -- tickets 056b1ee1, 2973e208
- Removing the per-epic max_workers override -- ticket 6e3f9e91

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T21:16Z | groomed | in_design | philippepascal |