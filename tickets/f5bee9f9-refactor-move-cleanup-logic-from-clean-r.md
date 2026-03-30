+++
id = "f5bee9f9"
title = "refactor: move cleanup logic from clean.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "philippepascal"
branch = "ticket/f5bee9f9-refactor-move-cleanup-logic-from-clean-r"
created_at = "2026-03-30T14:27:36.851282Z"
updated_at = "2026-03-30T16:31:35.753224Z"
+++

## Spec

### Problem

`clean.rs` contains 171 lines of cleanup detection and orchestration logic that
belongs in `apm-core`:

- Terminal state resolution (from config + hardcoded "closed")
- Merged branch detection via `git branch --merged`
- Ancestor check via `git merge-base --is-ancestor`
- State cross-check: ticket state on branch vs state on main
- Local vs remote tip agreement check
- Worktree dirty-check
- Local branch existence check

These are pure data checks on git state — not CLI concerns. `apm-serve` will
want to show a "ready to clean" list in the UI and trigger cleanup without
shelling out.

Target: `apm_core::clean::candidates()` returning branches safe to remove with
reasons, and `apm_core::clean::remove()` for the actual removal. CLI formats
and prompts.

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
| 2026-03-30T16:31Z | new | in_design | philippepascal |
