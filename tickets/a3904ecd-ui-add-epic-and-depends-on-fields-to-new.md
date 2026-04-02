+++
id = "a3904ecd"
title = "UI: add epic and depends_on fields to new ticket modal"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/a3904ecd-ui-add-epic-and-depends-on-fields-to-new"
created_at = "2026-04-01T21:56:06.583740Z"
updated_at = "2026-04-02T00:52:55.544496Z"
+++

## Spec

### Problem

The new ticket modal has no way to associate a ticket with an epic or declare dependencies. Without this, users cannot create epic-linked tickets from the UI — they must use the CLI.

The full design is in `docs/epics.md` (§ apm-ui changes — New ticket modal). Two optional fields are added below the title input:
- **Epic** — dropdown populated from `GET /api/epics`; selecting one pre-fills the epic ID
- **Depends on** — multi-value text input for ticket IDs, stored as `depends_on` array

Omitting both preserves the current free-ticket creation behaviour.

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
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:52Z | groomed | in_design | philippepascal |
