+++
id = "b6bc09d0"
title = "Refactor epic.rs: extract run_set ticket logic and apply shared helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b6bc09d0-refactor-epic-rs-extract-run-set-ticket-"
created_at = "2026-04-12T09:02:48.936896Z"
updated_at = "2026-04-12T09:28:51.684207Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f", "aeacd066"]
+++

## Spec

### Problem

`apm/src/cmd/epic.rs` (438 lines) is the largest command file and contains misplaced logic:

1. **`run_set()` for owner** (lines ~252-300) — when setting an epic's owner, this function iterates over all tickets in the epic and bulk-updates their `owner` field. This is ticket mutation logic that doesn't belong in the epic command module. It should be extracted to `apm_core::epic` as a function like `set_epic_owner(root, epic_id, owner)` that handles the cascading update.

2. **`run_close()` PR creation** (lines ~108-152) — contains inline `gh pr create` logic that's similar to `apm_core::state::gh_pr_create_or_update` (which was moved to `github.rs` in the apm-core refactoring epic). This should reuse the core function rather than reimplementing PR creation.

3. After the prerequisite ticket moves `branch_to_title()` and epic ID parsing to `apm_core::epic`, update this file to use the shared helpers instead of local definitions.

4. Apply shared `util.rs` helpers (from the prerequisite ticket) for any confirmation prompts or fetch patterns in this file.

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
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:28Z | groomed | in_design | philippepascal |
