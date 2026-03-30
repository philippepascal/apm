+++
id = "32a9a8b5"
title = "refactor: move sync candidate detection from sync.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-1640-spec1"
branch = "ticket/32a9a8b5-refactor-move-sync-candidate-detection-f"
created_at = "2026-03-30T14:27:39.762926Z"
updated_at = "2026-03-30T16:37:17.487398Z"
+++

## Spec

### Problem

`sync.rs` contains 172 lines of candidate detection logic that belongs in
`apm-core`:

- Merged branch detection (via `git::merged_into_main`)
- Accept candidate detection: implemented tickets whose branch is merged into main
- Close candidate detection: tickets in "accepted" state, or "implemented" on
  main with no ticket branch
- Squash-merge detection (via `git log --cherry-pick`)
- Batch accept/close orchestration

The interactive prompting (`[y/N]`) belongs in the CLI. The detection logic
does not. `apm-serve` will want to show a sync preview — "these tickets are
ready to accept/close" — without shelling out to `apm sync`.

Target: `apm_core::sync::detect()` returning structured candidates. CLI prompts
the user and calls `apm_core::sync::apply()`.

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
| 2026-03-30T16:34Z | new | in_design | philippepascal |
| 2026-03-30T16:37Z | 65590 | claude-0330-1640-spec1 | handoff |
