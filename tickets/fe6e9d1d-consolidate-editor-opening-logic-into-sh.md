+++
id = "fe6e9d1d"
title = "Consolidate editor-opening logic into shared CLI module"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/fe6e9d1d-consolidate-editor-opening-logic-into-sh"
created_at = "2026-04-07T22:30:48.429150Z"
updated_at = "2026-04-07T22:56:44.443948Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Editor-opening logic is duplicated across three command handlers with slight but meaningful variations:\n\n1. **`cmd/new.rs` lines 76–128** — resolves the editor, checks out the ticket branch, opens the editor on the ticket file, auto-commits (ignoring non-zero exit with a warning), then restores the original branch.\n2. **`cmd/show.rs` lines 83–130** — resolves the editor, writes ticket content to a temp file, opens it with inherited stdio, bails on non-zero exit, diffs the result, and commits via `git::commit_to_branch` if changed.\n3. **`cmd/review.rs` lines 158–180** — resolves the editor, opens it on an existing path with inherited stdio, bails on non-zero exit.\n\nAll three contain an identical block that reads `$VISUAL`, falls back to `$EDITOR`, then falls back to `"vi"`, and spawns the process by splitting the string on whitespace. This means any change to editor resolution or invocation (e.g., adding a new env var, changing error handling, adding logging) must be applied in three places, increasing the chance of divergence.

### Acceptance criteria

- [ ] `apm new` behaves identically to before — opens the editor, commits the result, restores the original branch
- [ ] `apm show --edit` behaves identically to before — opens the editor on a temp file, commits if the content changed
- [ ] `apm review` behaves identically to before — opens the editor on the review file and bails on non-zero exit
- [ ] Changing `$VISUAL` or `$EDITOR` at runtime is reflected in all three commands without touching cmd/ files
- [ ] When neither `$VISUAL` nor `$EDITOR` is set, all three commands fall back to `vi`
- [ ] `cargo test` passes with no new failures

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
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:56Z | groomed | in_design | philippepascal |