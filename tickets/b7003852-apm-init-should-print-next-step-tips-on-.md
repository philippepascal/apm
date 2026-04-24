+++
id = "b7003852"
title = "apm init should print next-step tips on completion"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b7003852-apm-init-should-print-next-step-tips-on-"
created_at = "2026-04-24T06:28:19.582833Z"
updated_at = "2026-04-24T06:28:19.582833Z"
+++

## Spec

### Problem

After apm init finishes, the only output is "apm initialized." (apm/src/cmd/init.rs:62). New users get no cue on what to do next: whether to commit the .apm/ changes, how to create a first ticket, or that apm-server exists. Expected: print a short tips block (~5 lines) before exit suggesting: (1) commit the .apm/ config files, (2) try apm new to create a first ticket, (3) try apm-server for the web UI, (4) use apm --help for the full CLI reference. Keep it brief; consider suppressing when stdin is not a tty (CI scripts) or behind a --quiet flag.

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
| 2026-04-24T06:28Z | — | new | philippepascal |
