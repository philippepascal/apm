+++
id = "48d3932b"
title = "Simplify apm prompt --explain output: hide cascade detail when no fallback fired"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/48d3932b-simplify-apm-prompt-explain-output-hide-"
created_at = "2026-05-30T07:40:46.558546Z"
updated_at = "2026-05-30T17:13:20.695474Z"
+++

## Spec

### Problem

`apm prompt --explain` currently produces confusing output in three ways. First, the layer-3 line includes parenthetical `(level N — label)` text that conflates cascade level numbers with the layer concept, forcing users to decode what "level" means vs. "layer". Second, a `skipped:` block appears at the same indent as the layer lines, making it look like a fourth layer rather than a sub-detail of layer 3. Third, even in the common case where the agent's own role file resolves immediately, two `not reached` lines are printed — noise that adds no information.

The desired output collapses to the minimum needed: show what was used, and when the cascade fell back, explain why. When the per-agent file exists, print its path on layer 3 with no cascade block. When one or both on-disk candidates were missing, show a single indented sub-line naming the path(s) that triggered the fallback.

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
| 2026-05-30T07:40Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:13Z | groomed | in_design | philippepascal |