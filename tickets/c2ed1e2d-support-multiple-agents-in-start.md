+++
id = "c2ed1e2d"
title = "support multiple agents in start"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c2ed1e2d-support-multiple-agents-in-start"
created_at = "2026-04-07T17:14:02.689742Z"
updated_at = "2026-04-07T17:47:54.483986Z"
+++

## Spec

### Problem

apm start, work, UI dispatcher currently on type of agent (claude by default). user should be able to start a different type of agent for spec writing and for implementation.
add a level of indirection in config: a user can create as many worker profiles as he wants. in our case we have spec_agent and impl_agent. They have their own configuration for how to spawn, and what instructions to use.
in workflow, we use these profiles for state transition instead of just the instructions.

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
| 2026-04-07T17:14Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:47Z | groomed | in_design | philippepascal |
