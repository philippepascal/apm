+++
id = "a56953f1"
title = "Add Context section to ticket format for delegator handoff"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
branch = "ticket/a56953f1-add-context-section-to-ticket-format-for"
created_at = "2026-04-01T22:09:53.033510Z"
updated_at = "2026-04-01T22:10:17.024845Z"
+++

## Spec

### Problem

When a delegator creates a ticket and promotes it to `groomed`, the spec-writer worker receives nothing beyond the ticket title. There is no sanctioned place in the ticket format for the delegator to record the relevant design document, the relevant section, or known constraints (e.g. "the `accepted` state has been removed").

The existing sections (`### Problem`, `### Acceptance criteria`, etc.) are worker-owned. Pre-filling them creates ambiguity about whether the worker should preserve or replace the content.

The result: spec-writers must guess intent from the title alone and often produce specs that miss the design or require amendment cycles.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T22:09Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:10Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-01T22:10Z | groomed | in_design | claude-0401-2145-a8f3 |