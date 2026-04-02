+++
id = "e8a56566"
title = "UI supervisor list APIs should not pull closed ticket by default"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
agent = "16802"
branch = "ticket/e8a56566-ui-supervisor-list-apis-should-not-pull-"
created_at = "2026-04-02T18:12:19.697833Z"
updated_at = "2026-04-02T18:13:05.178311Z"
+++

## Spec

### Problem

The `GET /api/tickets` endpoint currently returns all tickets unconditionally — including tickets in the `closed` state. The UI does all filtering client-side after receiving the full payload. Because closed tickets accumulate over time and can far outnumber active tickets, this causes progressively heavier API responses and slows the UI's fast-refresh polling loop.

The desired behaviour is that the server excludes closed (terminal) tickets from the default response, and only includes them when the caller explicitly opts in. This mirrors how the CLI already works: `apm list` hides terminal states unless `--all` is passed.

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
| 2026-04-02T18:12Z | — | new | apm-ui |
| 2026-04-02T18:12Z | new | groomed | apm |
| 2026-04-02T18:13Z | groomed | in_design | philippepascal |