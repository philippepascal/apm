+++
id = "d877bd37"
title = "Add epic, target_branch, and depends_on fields to ticket frontmatter"
state = "groomed"
priority = 10
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
branch = "ticket/d877bd37-add-epic-target-branch-and-depends-on-fi"
created_at = "2026-04-01T21:54:58.399434Z"
updated_at = "2026-04-01T21:59:09.021759Z"
+++

## Spec

### Problem

APM tickets currently have no way to express that they belong to a larger unit of work or that they depend on another ticket being completed first. Without these fields, all tickets are treated as independent, making it impossible to build epic-scoped workflows or enforce delivery ordering.

The full design is in `docs/epics.md` (§ Data model — Ticket frontmatter additions). Three new optional TOML frontmatter fields must be added to `TicketFrontmatter`:

- `epic = "<8-char-id>"` — associates the ticket with an epic branch
- `target_branch = "epic/<id>-<slug>"` — the branch the worktree and PR target (defaults to `main` when absent)
- `depends_on = ["<ticket-id>", ...]` — ticket IDs that must reach `implemented` before this ticket can be dispatched

All three fields are optional; omitting them preserves existing behaviour exactly. This ticket is the data-model foundation that all other epics tickets build on.

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
| 2026-04-01T21:54Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |