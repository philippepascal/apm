+++
id = "9c66e199"
title = "Unify worker command allow-list to six commands; remove per-role lists"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9c66e199-unify-worker-command-allow-list-to-six-c"
created_at = "2026-05-31T02:57:57.400665Z"
updated_at = "2026-05-31T07:33:30.439351Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

The `role_command_allowlist` function in `apm-core/src/instructions.rs` hard-codes two named roles — `"spec-writer"` and `"worker"` — each with its own 8-command list. Any role not matching those two strings (including `"coder"`) falls through to `None` and receives the full, unfiltered command list of 30+ entries. A coder agent today sees supervisor commands like `apm sync`, `apm list`, `apm next`, and `apm start` in its Command Reference — noise at best, misleading at worst.

Beyond the coder gap, the design is fragile: adding a new role requires editing match arms in code. Per project convention, role names belong in config, not in code. Both role lists also include supervisor-tier commands (`sync`, `list`, `next`) that workers should never invoke.

The fix is to replace the per-role match arms with a single `WORKER_COMMAND_ALLOWLIST` constant returned for any supplied role. The six commands — `show`, `state`, `spec`, `set`, `new`, `instructions` — cover everything a worker needs: reading the ticket, transitioning state, editing the spec, adjusting fields, filing side-notes, and bootstrapping the session. No role-name strings belong in the filtering logic after this change.

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
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:33Z | groomed | in_design | philippepascal |