+++
id = "9f4869d6"
title = "refactor: move review transition and body manipulation logic into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "85310"
branch = "ticket/9f4869d6-refactor-move-review-transition-and-body"
created_at = "2026-03-30T14:27:50.402284Z"
updated_at = "2026-03-30T16:35:41.862437Z"
+++

## Spec

### Problem

`review.rs` contains 321 lines mixing editor orchestration (CLI concern) with
business logic that belongs in `apm-core`:

- Manual transition detection from config (which transitions are supervisor-only)
- Spec body splitting (content vs history)
- Amendment section extraction from edited file
- Amendment section normalization (plain bullets → checkboxes)
- Transition validation against config
- Amendment request injection into ticket body

Only editor temp-file management and the `$VISUAL`/`$EDITOR`/`vi` invocation
belong in the CLI. Everything else is document manipulation that `apm-serve`
will need when a supervisor approves or requests amendments via the web UI —
without opening a local editor.

Target: `apm_core::review` module with `available_transitions()`,
`apply_review()`, `normalize_amendments()`. CLI handles editor and calls these.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:35Z | new | in_design | philippepascal |