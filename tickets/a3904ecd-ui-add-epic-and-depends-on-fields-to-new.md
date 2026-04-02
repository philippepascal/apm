+++
id = "a3904ecd"
title = "UI: add epic and depends_on fields to new ticket modal"
state = "in_design"
priority = 2
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "8475"
branch = "ticket/a3904ecd-ui-add-epic-and-depends-on-fields-to-new"
created_at = "2026-04-01T21:56:06.583740Z"
updated_at = "2026-04-02T00:52:55.544496Z"
+++

## Spec

### Problem

The new ticket modal offers only a title and four spec-section text areas. There is no way to associate a ticket with an epic or declare dependencies from the UI — users must resort to the CLI. This blocks teams that prefer managing epic-linked work through the web interface.

The full design is in `docs/epics.md` (§ apm-ui changes — New ticket modal). Two optional fields are added below the title input:
- **Epic** — dropdown populated from `GET /api/epics`; selecting one includes the short epic ID in the create payload.
- **Depends on** — free-text input for space- or comma-separated ticket IDs, stored as a `depends_on` array.

Neither `GET /api/epics` nor the `epic` / `depends_on` frontmatter fields exist yet. This ticket covers the UI modal changes plus the minimum server and core changes required to make those fields functional.

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