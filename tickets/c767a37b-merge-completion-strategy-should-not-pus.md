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

when doing an apm state with a completion = "merge", apm should not do the final push to origin main. this last step is a supervisor action.

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