+++
id = "f5eda44b"
title = "UI: show epic and depends_on in ticket detail panel"
state = "groomed"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
branch = "ticket/f5eda44b-ui-show-epic-and-depends-on-in-ticket-de"
created_at = "2026-04-01T21:56:10.584818Z"
updated_at = "2026-04-01T22:01:02.029614Z"
+++

## Spec

### Problem

The ticket detail panel shows the ticket's core fields but not `epic` or `depends_on`. Engineers inspecting a ticket cannot see which epic it belongs to or which tickets it is waiting on.

The full design is in `docs/epics.md` (§ apm-ui changes — Ticket detail panel). When `epic` is present, show a clickable label that sets the epic filter on the supervisor board. When `depends_on` is present, show a list of ticket IDs — each links to that ticket's detail panel, and resolved tickets (state `implemented` or later) are shown with strikethrough.

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