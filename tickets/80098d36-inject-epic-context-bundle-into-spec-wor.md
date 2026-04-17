+++
id = "80098d36"
title = "Inject epic context bundle into spec workers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/80098d36-inject-epic-context-bundle-into-spec-wor"
created_at = "2026-04-17T07:27:05.870212Z"
updated_at = "2026-04-17T07:33:26.830012Z"
epic = "35199c7f"
target_branch = "epic/35199c7f-give-workers-cross-ticket-context"
+++

## Spec

### Problem

When a spec worker is spawned (at `in_design`, from either `groomed` or `ammend`), it sees only its own ticket. It doesn't know what the broader epic is trying to accomplish or what sibling tickets are claiming. As a result, specs drift out of scope, duplicate work that a sibling will do, or miss acceptance criteria that are only obvious from the epic's shape. This is the dominant cause of amendment cycles during the spec phase.

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
| 2026-04-17T07:27Z | — | new | philippepascal |
| 2026-04-17T07:33Z | new | groomed | claude-0417-1430-c7a2 |
| 2026-04-17T07:33Z | groomed | in_design | claude-0417-1430-c7a2 |