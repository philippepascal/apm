+++
id = "ab1eb252"
title = "Improve apm epic close UX: help text, auto-sync mergeable tickets, --merge/--pr/--auto"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab1eb252-improve-apm-epic-close-ux-help-text-auto"
created_at = "2026-05-30T18:53:24.160398Z"
updated_at = "2026-05-30T19:01:47.036066Z"
+++

## Spec

### Problem

Three improvements to apm epic close (apm/src/cmd/epic.rs::run_close, lines 73-132):

1. Help text should briefly list the high-level steps the command performs: quiescence check, push epic branch, create or update PR, with a note that the branch is just deleted (no PR) when it is already merged into default. Today the help is one sentence and users do not know what the command is about to do.

2. When the quiescence check fails because tickets in the epic are still in non-closed states, the command should not just bail with the blocker list. It should detect tickets whose branches are already merged into the epic branch or the default branch and offer to close them automatically, the same way apm sync already prompts to close merged tickets. Tickets that genuinely need manual attention should still be listed as blockers; tickets that are merely waiting for the closing transition should be offered for auto-close.

3. Add --merge, --pr, and --auto flags mirroring the pattern already used by apm epic refresh (run_refresh_epic). Semantics:
   --merge does a working-tree merge of the epic branch into default and skips PR creation
   --pr (the current default behaviour) pushes the epic branch and opens or updates a PR
   --auto merges when the merge would be clean; falls back to opening a PR when it would conflict
The current default (push + open PR) should remain the default when no flag is given.

Reference: run_refresh_epic in apm/src/cmd/epic.rs already implements the --merge/--pr/--auto pattern and the merge_tree_status helper that distinguishes clean vs conflicted merges. The new flags on run_close should reuse the same helpers, not duplicate the logic.

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
| 2026-05-30T18:53Z | — | new | philippepascal |
| 2026-05-30T18:57Z | new | groomed | philippepascal |
| 2026-05-30T19:01Z | groomed | in_design | philippepascal |
