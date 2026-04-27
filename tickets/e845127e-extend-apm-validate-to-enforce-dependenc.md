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
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T21:16Z | groomed | in_design | philippepascal |