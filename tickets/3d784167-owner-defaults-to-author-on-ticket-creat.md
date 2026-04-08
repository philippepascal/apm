+++
id = "3d784167"
title = "Owner defaults to author on ticket creation"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/3d784167-owner-defaults-to-author-on-ticket-creat"
created_at = "2026-04-08T15:09:41.414576Z"
updated_at = "2026-04-08T15:09:41.414576Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

When a ticket is created with `apm new`, the `owner` field is not set (or set to empty/None). Per the ownership spec, owner should default to the author (the current user creating the ticket). This ensures the creator has immediate control over the ticket and can assign it to others or dispatch workers against it.

### Acceptance criteria

- [ ] `apm new` sets `owner` = `author` (from `resolve_identity()`) in the ticket frontmatter
- [ ] Tickets created without explicit owner have owner == author in the persisted markdown
- [ ] `apm show <id>` displays the owner field
- [ ] `apm list` output includes the owner column
- [ ] Existing tickets without owner field still parse (owner defaults to None/empty)
- [ ] Tests cover owner-on-creation behavior

### Out of scope

Owner validation against collaborators (separate tickets). Changing owner after creation (separate ticket).

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |