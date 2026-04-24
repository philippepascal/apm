+++
id = "c706c825"
title = "Default agents.md should forbid main agent from grooming"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c706c825-default-agents-md-should-forbid-main-age"
created_at = "2026-04-24T06:28:59.221174Z"
updated_at = "2026-04-24T06:28:59.221174Z"
+++

## Spec

### Problem

The default apm-core/src/default/apm.agents.md lets the main/delegator agent continue the state machine past new — including new -> groomed — when creating tickets, even though grooming is the supervisor review gate. Observed on ticker: main agent routinely created AND groomed tickets in a single pass, skipping supervisor review. Fix already proven in ticker repo at /Users/philippepascal/repos/ticker/.apm/agents.md, which adds a "Supervisor-only transitions" section listing states the main agent MUST NOT advance: new -> groomed, specd -> ready / ammend, implemented -> ready / ammend / closed, blocked -> ready, any "apm epic close". Expected: port that block into the default agents.md template. Related to the "project-agnostic defaults" ticket — that should land first; this adds the supervisor-only section on top.

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
