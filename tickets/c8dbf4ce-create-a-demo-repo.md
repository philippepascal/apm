+++
id = "c8dbf4ce"
title = "create a demo repo"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8dbf4ce-create-a-demo-repo"
created_at = "2026-04-07T17:01:04.559759Z"
updated_at = "2026-04-07T17:43:43.554278Z"
+++

## Spec

### Problem

a public repo called apm-demo, with a simple "dummy but functional software" (like a simple rust command line that just outputs some text).
it uses apm, apm is preinstalled (but assumes the user has installed the binaries)
It's frozen in middle of development, but compile and runs.
It has tickets in all possible states and combinations (or at least a representative subset), but making sense in the context of the project. it should touch all features (tickets, epics, depends, default branch, merge strategy, etc)
I allows someone to clone it, use apm to "kick the tires".
The readme runs a user through next steps to learn how to use apm. apm init if needed (or just mentionned it was already run), apm commands, apm-server, etc

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
| 2026-04-07T17:01Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |
