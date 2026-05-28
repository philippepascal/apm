+++
id = "d2720f0b"
title = "apm new editor flow must not checkout the ticket branch in main"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d2720f0b-apm-new-editor-flow-must-not-checkout-th"
created_at = "2026-05-28T07:37:16.051173Z"
updated_at = "2026-05-28T07:39:25.447173Z"
depends_on = ["f16e4035"]
+++

## Spec

### Problem

`apm new` (without `--no-edit`) currently checks out the ticket branch in the main working tree so the ticket file lands on disk for the editor. During the editor session, `HEAD` points to the ticket branch rather than the branch the user was on before. This side effect is what allowed `find_worktree_for_branch` (before f16e4035) to return the main repo path when the ticket branch was checked out there, triggering incorrect dispatch. Even after f16e4035, the checkout-based flow is undesirable: it moves HEAD for the duration of an interactive session (potentially minutes), blocking any concurrent read of the main repo's branch state, and it makes `--no-edit` a safety requirement for agents rather than a performance flag.

The desired behaviour is: read the ticket file from the ticket branch via git plumbing, write it to a temp file, open the editor on that temp file, read the result back, and commit it to the ticket branch using `commit_to_branch` — which already handles temp worktrees and never touches HEAD. The main working tree is never modified.

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
| 2026-05-28T07:37Z | — | new | philippepascal |
| 2026-05-28T07:37Z | new | groomed | philippepascal |
| 2026-05-28T07:39Z | groomed | in_design | philippepascal |