+++
id = "da95246d"
title = "UI: show lock icon on ticket cards with unresolved depends_on"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "64122"
branch = "ticket/da95246d-ui-show-lock-icon-on-ticket-cards-with-u"
created_at = "2026-04-01T21:56:15.495249Z"
updated_at = "2026-04-02T00:54:39.595316Z"
+++

## Spec

### Problem

Ticket cards on the supervisor board give no visual signal when a ticket is waiting on unresolved `depends_on` entries. An engineer looking at the board cannot tell at a glance which tickets are blocked by dependencies and why they are not being dispatched.

The `depends_on` field is not yet part of the `Frontmatter` struct in `apm-core`, so no dependency data is tracked or surfaced through the API. Adding it and exposing it to the UI unlocks this and future dependency-aware features.

The desired behaviour (per `docs/epics.md` § "Ticket cards") is: cards where `depends_on` has at least one unresolved entry show a small lock icon; hovering the icon shows a tooltip listing the blocking ticket IDs and their current states.

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
| 2026-04-01T21:56Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:01Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:54Z | groomed | in_design | philippepascal |