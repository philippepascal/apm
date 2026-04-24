+++
id = "c706c825"
title = "Default agents.md should forbid main agent from grooming"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c706c825-default-agents-md-should-forbid-main-age"
created_at = "2026-04-24T06:28:59.221174Z"
updated_at = "2026-04-24T07:17:37.533240Z"
+++

## Spec

### Problem

The default `apm-core/src/default/apm.agents.md` template contains no restriction preventing the main/delegator agent from running state transitions that are reserved for the supervisor. Without this guardrail, the main agent can create a ticket and immediately advance it through states that are supposed to be supervisor review gates — including `new → groomed`, which is where the supervisor decides whether a ticket is worth speccing. This was observed in practice in the ticker repo: the main agent routinely created *and* groomed tickets in a single pass, so the supervisor never had a chance to reject or defer them.

The fix is already proven. The ticker repo's `.apm/agents.md` adds a **Supervisor-only transitions** paragraph to the `### Main Agent` section, listing every transition the main agent must never run and the narrow set it may initiate itself. That paragraph needs to be ported verbatim into the default template so every project initialized with `apm init` gets the guardrail automatically.

This ticket depends on ticket 10791dab ("Default apm init templates should be project-agnostic"), which restructures the same file. This change adds a new content block; 10791dab should land first to avoid a merge conflict.

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
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:17Z | groomed | in_design | philippepascal |