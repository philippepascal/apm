+++
id = "2eec2b83"
title = "Refactor clean.rs: extract epic cleanup and apply shared helpers"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2eec2b83-refactor-clean-rs-extract-epic-cleanup-a"
created_at = "2026-04-12T09:02:46.720913Z"
updated_at = "2026-04-12T09:02:46.720913Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f", "aeacd066"]
+++

## Spec

### Problem

`apm/src/cmd/clean.rs` (296 lines) mixes two unrelated responsibilities:

1. **Local worktree/branch cleanup** (`run()`, ~70 lines) — removes worktrees and branches for closed tickets. This is appropriate for `clean.rs`.

2. **Epic cleanup** (`run_epic_clean()`, ~120 lines) — lists epic branches, prompts for deletion, removes branches, and cleans up `.apm/epics.toml`. This function:
   - Duplicates epic ID parsing logic that exists in `epic.rs` (and should be in `apm_core::epic` after the prerequisite ticket)
   - Mixes git branch operations with TOML file manipulation
   - Contains its own user interaction prompts (should use shared `util::prompt_yes_no()` after the prerequisite ticket)

`run_epic_clean()` should either move to `epic.rs` (since it's epic-domain logic) or become a function in `apm_core::clean`/`apm_core::epic` with the CLI command just handling user prompts. The confirmation prompts should use the shared utility from `util.rs`.

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
| 2026-04-12T09:02Z | — | new | philippepascal |