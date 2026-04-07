+++
id = "ce919ea8"
title = "Unify state transition logic into single module"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/ce919ea8-unify-state-transition-logic-into-single"
created_at = "2026-04-07T22:30:50.389099Z"
updated_at = "2026-04-07T22:30:50.389099Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
depends_on = ["eea2c9bc"]
+++

## Spec

### Problem

State transition logic is scattered across four modules in apm-core:

- `state.rs` — the core `transition()` function: validates target state, updates frontmatter, appends history, handles completion strategy (merge/PR)
- `start.rs` — `run()` transitions to `in_progress`, provisions worktrees, spawns workers; duplicates some of the transition validation from `state.rs`
- `ticket.rs` — `close()` handles terminal transitions, duplicates history appending
- `review.rs` — `apply_review()` manipulates spec body during review transitions, including amendment normalization

The transition engine in `state.rs` is the canonical path, but `start.rs` and `ticket.rs` bypass it for their specific transitions. This means:
- History appending logic exists in both `state.rs` and `ticket.rs`
- Post-transition side effects (worktree provisioning, PR creation, merge) are handled inline in `state.rs` rather than being composable
- Adding a new transition behavior requires understanding which of the four files handles that particular state change

A contributor modifying transition behavior must read all four files to understand the full picture.

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
| 2026-04-07T22:30Z | — | new | philippepascal |