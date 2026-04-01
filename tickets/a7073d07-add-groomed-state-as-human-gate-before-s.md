+++
id = "a7073d07"
title = "Add groomed state as human gate before spec work"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/a7073d07-add-groomed-state-as-human-gate-before-s"
created_at = "2026-04-01T20:26:40.952240Z"
updated_at = "2026-04-01T20:26:40.952240Z"
+++

## Spec

### Problem

Currently agents pick up 'new' tickets directly for spec writing, with no human triage gate. We need a 'groomed' state between new and in_design that acts as a human approval gate before spec work begins — mirroring how 'ready' gates implementation work. Changes needed: add 'groomed' to apm.toml workflow states with actionable=["agent"] for spec pickup; remove agent actionability from 'new'; update agents.md to reflect that agents pick up 'groomed' not 'new' tickets; update apm.toml transition from new->groomed (supervisor/engineer actor) and groomed->in_design (agent actor). Tickets start in 'new' (no change to apm new command). The delegator dispatches groomed tickets to spec agents, not new tickets.

What is broken or missing, and why it matters.

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
| 2026-04-01T20:26Z | — | new | apm |
