+++
id = "78eeb755"
title = "Rewrite apm.worker.md built-in default"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/78eeb755-rewrite-apm-worker-md-built-in-default"
created_at = "2026-05-22T23:22:24.735576Z"
updated_at = "2026-05-23T00:06:33.529498Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

apm.worker.md built-in default (apm-core/src/default/agents/default/apm.worker.md) currently contains shell discipline verbatim — this will move to apm instructions (see T1). It also contains role-detection language that is no longer needed. Remove both. Add History/Filename preservation rules that currently exist only in apm.spec-writer.md (workers also edit ticket files and need these rules). Net result: shorter, cleaner role file that only describes worker/implementer behavior. Do NOT touch the claude/ override in this ticket — that is T6.

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
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:06Z | groomed | in_design | philippepascal |
