+++
id = "c8d5590d"
title = "correct mispelling ammend->amend in workflow and anywhere else it might be"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8d5590d-correct-mispelling-ammend-amend-in-workf"
created_at = "2026-06-16T18:18:48.186548Z"
updated_at = "2026-06-16T18:19:36.620606Z"
+++

## Spec

### Problem

The workflow state intended to request spec or implementation revisions is named `ammend` throughout the codebase — in the workflow TOML, Rust source, tests, agent instructions, and documentation. The correct English spelling is `amend`. The misspelling propagated from the initial workflow definition and was copied into every layer that references the state by name.

Because the state ID is a bare string used in comparisons, config files, TOML fixtures, and user-facing help text, the misspelling appears in the interface agents and supervisors see every time they interact with this state. Fixing it corrects the language without changing any behaviour.

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
| 2026-06-16T18:18Z | — | new | philippepascal |
| 2026-06-16T18:19Z | new | groomed | philippepascal |
| 2026-06-16T18:19Z | groomed | in_design | philippepascal |