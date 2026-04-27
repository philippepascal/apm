+++
id = "e845127e"
title = "Extend apm validate to enforce dependency rules across tickets"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e845127e-extend-apm-validate-to-enforce-dependenc"
created_at = "2026-04-27T20:28:41.454959Z"
updated_at = "2026-04-27T20:28:41.454959Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["a3dc64db"]
+++

## Spec

### Problem

`apm validate` (see `apm validate --help`) currently checks apm.toml correctness, branch-field consistency, and uniqueness of branch names. It does not check that existing tickets' `depends_on` satisfies the strategy rules from the spec at `docs/strategy-and-dependencies.md` (section 'Dependency rules per strategy').

Extend `apm validate` to walk every ticket and report any whose `depends_on` violates the current rule (per the configured completion strategy and the deps' epic / target_branch fields). Failures appear in both human and `--json` output, with one entry per violating ticket.

Reuse the helper added in ticket a3dc64db (strategy-aware dependency rules) so the rule lives in exactly one place and `apm validate`, `apm new`, and `apm start` enforce identical semantics.

See docs/strategy-and-dependencies.md, section 'Dependency rules per strategy'.

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
