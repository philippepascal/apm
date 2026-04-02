+++
id = "d877bd37"
title = "Add epic, target_branch, and depends_on fields to ticket frontmatter"
state = "in_design"
priority = 10
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "50689"
branch = "ticket/d877bd37-add-epic-target-branch-and-depends-on-fi"
created_at = "2026-04-01T21:54:58.399434Z"
updated_at = "2026-04-02T00:43:05.975391Z"
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

- [ ] A ticket file with `epic = "ab12cd34"` in frontmatter parses without error and `ticket.frontmatter.epic` equals `"ab12cd34"`
- [ ] A ticket file with `target_branch = "epic/ab12cd34-user-auth"` in frontmatter parses without error and `ticket.frontmatter.target_branch` equals `"epic/ab12cd34-user-auth"`
- [ ] A ticket file with `depends_on = ["cd56ef78", "12ab34cd"]` in frontmatter parses without error and `ticket.frontmatter.depends_on` equals `["cd56ef78", "12ab34cd"]`
- [ ] A ticket file with none of the three new fields parses without error, with all three fields absent/None (backward-compatible)
- [ ] Serialising a ticket whose `epic`, `target_branch`, and `depends_on` fields are absent produces no mention of those keys in the TOML output
- [ ] `pick_next` skips a ticket whose `depends_on` list contains at least one ID that corresponds to a ticket not yet in `implemented` or a terminal state
- [ ] `pick_next` returns a ticket whose `depends_on` entries are all in `implemented` or a terminal state
- [ ] `pick_next` does not skip a ticket whose `depends_on` references an ID that matches no known ticket (unknown dependency is treated as non-blocking)
- [ ] `apm state <id> implemented` opens the PR against `target_branch` when that field is set, instead of the configured default branch

### Out of scope

- `apm epic` subcommands (new, list, show, close) — covered by a separate ticket
- `apm new --epic` flag and epic-aware ticket creation — separate ticket
- apm-server epic API routes (`GET/POST /api/epics`) — separate ticket
- apm-ui epic filter, ticket card lock icon, and engine epic selector — separate ticket
- `apm work --epic` exclusive-mode flag — separate ticket
- `apm epic sync` / merging epic branches — explicitly not planned
- Validation that `epic` and `target_branch` are consistent with each other

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
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |