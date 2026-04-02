+++
id = "f5eda44b"
title = "UI: show epic and depends_on in ticket detail panel"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "23932"
branch = "ticket/f5eda44b-ui-show-epic-and-depends-on-in-ticket-de"
created_at = "2026-04-01T21:56:10.584818Z"
updated_at = "2026-04-02T00:53:14.903037Z"
+++

## Spec

### Problem

The ticket detail panel (`apm-ui/src/components/TicketDetail.tsx`) renders core fields — title, state, effort, risk, priority — but has no awareness of `epic` or `depends_on`. Engineers cannot tell from the UI which epic a ticket belongs to, or which tickets it is waiting on before it can be dispatched.

The underlying `Frontmatter` struct in `apm-core/src/ticket.rs` does not yet declare `epic` or `depends_on` fields, so they are stripped during parsing even when present in the TOML frontmatter. Adding them to the struct is the minimal server-side change needed; the `TicketDetailResponse` already flattens `Frontmatter`, so the new fields will appear in the API automatically.

On the UI side, two small features are needed in the detail panel header: a clickable epic label (that sets the epic filter on the supervisor board so engineers can quickly scope to the same epic) and a dependency list (ticket IDs that link to each dep's detail panel, with strikethrough on resolved deps).

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
| 2026-04-02T00:53Z | groomed | in_design | philippepascal |