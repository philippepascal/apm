+++
id = "ba4e8499"
title = "Add epic and depends_on fields to CreateTicketRequest and ticket API responses"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/ba4e8499-add-epic-and-depends-on-fields-to-create"
created_at = "2026-04-01T21:55:57.801343Z"
updated_at = "2026-04-02T00:43:44.702702Z"
+++

## Spec

### Problem

The ticket creation API and ticket response types do not include `epic`, `target_branch`, or `depends_on`. Without these, the UI cannot create epic-linked tickets via the API, and ticket list/detail responses omit epic membership information.

The full design is in `docs/epics.md` (§ apm-server changes — CreateTicketRequest and Ticket routes). `CreateTicketRequest` gains two new optional fields: `epic: Option<String>` and `depends_on: Option<Vec<String>>`. When `epic` is set, the server resolves `target_branch` from the epic branch name before calling `apm new`. For `TicketResponse` and `TicketDetailResponse`, the new frontmatter fields appear automatically via `#[serde(flatten)]` — no struct changes required for read paths.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
