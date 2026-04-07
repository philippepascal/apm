+++
id = "fe6e9d1d"
title = "Consolidate editor-opening logic into shared CLI module"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/fe6e9d1d-consolidate-editor-opening-logic-into-sh"
created_at = "2026-04-07T22:30:48.429150Z"
updated_at = "2026-04-07T22:30:48.429150Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Editor-opening logic is duplicated in three command handlers with slight variations:

1. `cmd/new.rs:76-128` — checks out ticket branch, opens editor on the ticket file, commits changes, restores original branch
2. `cmd/show.rs:83-130` — writes ticket to temp file, opens editor, reads back, diffs for changes, commits if modified
3. `cmd/review.rs:158-180` — opens editor with stdio inheritance, reads result back

All three resolve `$EDITOR` / `$VISUAL`, handle the fallback to `vi`, spawn the process, and check the exit code. The differences are in what happens before and after the editor runs (branch management, temp files, commit logic).

This makes it error-prone to change editor behavior (e.g., adding a new env var fallback, changing error handling) since the fix must be applied in three places.

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