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

- [ ] After `apm new <title>` (without `--no-edit`) completes, `git branch --show-current` in the main repo returns the same branch that was checked out before the command ran.
- [ ] The content shown in the editor is the content committed on the ticket branch at the moment the editor launches (not an empty file or stale content).
- [ ] Changes made in the editor are committed to the ticket branch at `tickets/<id>-<slug>.md` with the commit message `write spec`.
- [ ] `git checkout` is never invoked against the main repo during the editor session — no `git checkout <ticket-branch>` or `git checkout <prev-branch>` calls are made.
- [ ] The temp file created for editing is removed after the editor exits (best-effort; a removal failure must not fail the command).
- [ ] `--no-edit` is unaffected: when passed, no editor opens, no temp file is created, and HEAD is never moved.

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