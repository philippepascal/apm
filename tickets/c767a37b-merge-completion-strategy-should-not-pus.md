+++
id = "c767a37b"
title = "Merge completion strategy should not push main to origin"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "4076"
branch = "ticket/c767a37b-merge-completion-strategy-should-not-pus"
created_at = "2026-04-02T03:15:29.694878Z"
updated_at = "2026-04-02T16:56:03.666663Z"
+++

## Spec

### Problem

When a state transition with `completion = "merge"` is executed (e.g. `apm state <id> implemented`), the merge completion strategy performs five steps:\n\n1. Push the ticket branch to origin\n2. Fetch the default branch from origin\n3. Find the correct merge directory\n4. Merge the ticket branch into the default branch locally\n5. **Push the default branch to origin**\n\nStep 5 is the problem. Pushing `main` (or the configured default branch) to origin is a supervisor action — it publishes the merged work publicly and is a destructive, non-reversible operation. An autonomous agent completing a ticket should not have this authority. The push should be left to the human supervisor, who can review the local merge state before deciding to publish.

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
| 2026-04-02T03:15Z | — | new | apm |
| 2026-04-02T16:55Z | new | groomed | apm |
| 2026-04-02T16:56Z | groomed | in_design | philippepascal |