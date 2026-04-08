+++
id = "56cf5dca"
title = "Push ticket branches to origin on creation when aggressive sync is enabled"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/56cf5dca-push-ticket-branches-to-origin-on-creati"
created_at = "2026-04-08T15:40:56.947438Z"
updated_at = "2026-04-08T15:40:56.947438Z"
+++

## Spec

### Problem

When `apm new` creates a ticket, the branch is only created locally. In a multi-user setup, other collaborators and the server cannot see the ticket until the branch is pushed to origin. This breaks the collaborative workflow: a supervisor creates tickets and grooms them, but no one else sees them.

Additionally, state transitions fetch dependency branches from origin (`git fetch origin <branch>`), producing noisy `fatal: couldn't find remote ref` errors in server logs when those branches are local-only. Pushing on creation would eliminate this noise.

When `aggressive = true` in sync config, `apm new` (and other branch-creating commands like `apm epic new`) should push the branch to origin immediately after creation. This matches the aggressive sync philosophy: keep local and remote in sync at all times.

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
| 2026-04-08T15:40Z | — | new | philippepascal |