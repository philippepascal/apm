+++
id = "6cf21715"
title = "apm verify should detect missing worktree for active-state tickets"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cf21715-apm-verify-should-detect-missing-worktre"
created_at = "2026-04-28T00:50:59.455196Z"
updated_at = "2026-04-28T00:50:59.455196Z"
+++

## Spec

### Problem

`apm verify` does not detect the case where a ticket is in an active state (`in_design` or `in_progress`) but its worktree directory is missing from disk. This leaves tickets in an unrecoverable state without a clear diagnostic.

Real incident: ticket ec5e9fe3 was in `in_progress`. `apm worktrees` listed an entry for it at `…/apm--worktrees/ticket-ec5e9fe3-add-apm-spec-append-and-add-task-for-non`. The directory did not exist on disk. `apm verify` ran cleanly and did not flag the mismatch.

Expected behavior: `apm verify` walks every ticket whose state is in `{in_design, in_progress}` (i.e. states that imply a live worktree), resolves the expected worktree path for the ticket's branch, and reports an issue when the directory is missing.

Out-of-band states like `groomed`, `specd`, `ready`, `blocked`, `implemented`, `closed` do not require a worktree — only the explicitly active states do.

Fix direction: in `apm/src/cmd/verify.rs` (or wherever the verify check lives), after the existing branch/frontmatter checks, add a worktree-presence check for active-state tickets. Report "ticket X is in_progress but worktree at <path> is missing" with kind "worktree_missing". `--fix` should NOT auto-recreate the worktree, since recreation would silently lose any uncommitted work that may exist in another clone — report only, and let the supervisor decide whether to re-provision via `apm start <id>` or revert state to `ready`.

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
| 2026-04-28T00:50Z | — | new | philippepascal |
