+++
id = "ba4e8499"
title = "Add epic and depends_on fields to CreateTicketRequest and ticket API responses"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "85628"
branch = "ticket/ba4e8499-add-epic-and-depends-on-fields-to-create"
created_at = "2026-04-01T21:55:57.801343Z"
updated_at = "2026-04-02T00:43:44.702702Z"
+++

## Spec

### Problem

The `CreateTicketRequest` struct in `apm-server/src/main.rs` accepts only `title` and `sections`. It has no `epic` or `depends_on` fields, so the UI cannot create epic-linked or dependency-declared tickets via the API.

The `Frontmatter` struct in `apm-core/src/ticket.rs` also has no `epic`, `target_branch`, or `depends_on` fields. Because `TicketResponse` and `TicketDetailResponse` both serialize frontmatter via `#[serde(flatten)]`, adding these fields to `Frontmatter` is sufficient to make them appear in all existing ticket API read responses — no struct changes are needed in `apm-server`.

The `ticket::create` function must also be extended to accept and persist these three optional fields so the server (and the CLI in a future ticket) can populate them at creation time.

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