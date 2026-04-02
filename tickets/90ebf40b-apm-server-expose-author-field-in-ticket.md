+++
id = "90ebf40b"
title = "apm-server: expose author field in ticket API responses"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "65291"
branch = "ticket/90ebf40b-apm-server-expose-author-field-in-ticket"
created_at = "2026-04-02T20:54:08.576527Z"
updated_at = "2026-04-02T23:42:21.422012Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["610be42e"]
+++

## Spec

### Problem

The `author` field exists in `Frontmatter` but is declared `#[serde(skip_serializing_if = "Option::is_none")]`. Tickets that lack an `author` value (e.g. created before ticket #610be42e lands, or test fixtures) produce JSON responses with no `author` key at all. The UI cannot reliably read, display, or filter by ticket ownership when the field may be absent.

Two additional gaps compound this: (1) `GET /api/tickets` has no `author` query parameter, so the UI must download and filter all tickets client-side; (2) there is no `GET /api/me` endpoint, so the supervisor board cannot know whose tickets to show by default.

Together these gaps block the supervisor-board author filter and the per-author default view described in DESIGN-users.md points 1 and 8.

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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:22Z | new | groomed | apm |
| 2026-04-02T23:42Z | groomed | in_design | philippepascal |