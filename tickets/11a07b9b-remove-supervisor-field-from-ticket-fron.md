+++
id = "11a07b9b"
title = "Remove supervisor field from ticket frontmatter"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/11a07b9b-remove-supervisor-field-from-ticket-fron"
created_at = "2026-04-08T15:09:32.454090Z"
updated_at = "2026-04-08T15:09:32.454090Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
+++

## Spec

### Problem

The ticket frontmatter has a `supervisor` field that is not part of the ownership model. The ownership spec defines only two fields: `author` (immutable, who created) and `owner` (who manages). The `supervisor` field adds confusion and overlaps with `owner`. It must be removed from the Frontmatter struct, all set_field handling, list filters, and any UI references.

### Acceptance criteria

- [ ] `supervisor` field removed from `Frontmatter` struct in `ticket.rs`
- [ ] `apm set <id> supervisor` no longer accepted (returns error)
- [ ] `apm list --supervisor` filter removed
- [ ] Any server/UI references to supervisor field removed
- [ ] Existing tickets with supervisor field still parse without error (field ignored)
- [ ] All tests pass

### Out of scope

Migrating existing tickets that have supervisor set. They will simply be ignored on parse.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |