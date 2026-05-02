+++
id = "6095305a"
title = "Filesystem path validator at wrapper layer (worktree isolation enforcement)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6095305a-filesystem-path-validator-at-wrapper-lay"
created_at = "2026-05-01T02:30:34.552318Z"
updated_at = "2026-05-02T03:21:24.434090Z"
+++

## Spec

### Problem

Workers spawn inside a dedicated git worktree (`APM_TICKET_WORKTREE`) and are
expected to confine all filesystem writes to that tree. Two gaps make this
expectation unenforceable today:

1. **`-P` workers have no write boundary.** Spawning with
   `--dangerously-skip-permissions` bypasses Claude Code's permission allowlist
   entirely. Such a worker can `Write`, `Edit`, or `Bash`-redirect to any path
   on the filesystem — including the main worktree, other ticket worktrees, or
   paths outside the repo.

2. **Default-permission workers have accidental coverage leaks.** Explicit
   allowlist entries added for legitimate APM paths (e.g. `apm spec` temp
   files, `.apm/` directories) share a prefix with the actual repo root, which
   inadvertently permits writes to the main worktree as well.

The wrapper epic (4312fbd4) introduces an interception layer that sees every
tool-call event before it is dispatched. That is the correct surface for
enforcement: check the target path before the tool executes, and inject a
synthetic `tool_result` error if the path violates policy.

This ticket wires a `PathGuard` into that interception hook for the
`claude` built-in wrapper, adds a per-manifest opt-in field for custom
wrappers, and backs the allow-list with a project-level config section.

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
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:21Z | groomed | in_design | philippepascal |