+++
id = "34ad9126"
title = "Rewrite apm.spec-writer.md built-in default"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/34ad9126-rewrite-apm-spec-writer-md-built-in-defa"
created_at = "2026-05-22T23:22:22.098663Z"
updated_at = "2026-05-22T23:50:43.725043Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

apm.spec-writer.md built-in default (apm-core/src/default/agents/default/apm.spec-writer.md, 272 lines) currently contains shell discipline verbatim — this will move to apm instructions (see T1). It also contains role-detection language ("if your prompt contains X you are a worker") which is no longer needed once prompt assembly selects the right role file per agent. Remove both. Also: History/Filename preservation rules (how to preserve ## History and frontmatter when editing tickets) exist here but not in apm.worker.md — keep them. Net result: shorter, cleaner role file that only describes spec-writer behavior. The built-in claude/apm.spec-writer.md override (apm-core/src/default/agents/claude/apm.spec-writer.md) should be reviewed for any meaningful delta vs the default and merged or deleted.

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
