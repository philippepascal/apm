+++
id = "3d784167"
title = "Owner defaults to author on ticket creation"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
branch = "ticket/3d784167-owner-defaults-to-author-on-ticket-creat"
created_at = "2026-04-08T15:09:41.414576Z"
updated_at = "2026-04-08T21:57:54.487367Z"
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

All changes are in Rust. No schema migrations needed — owner is already Option<String> in Frontmatter and serializes when Some.

**1. Set owner on creation — apm-core/src/ticket.rs ~line 473**

In create(), the Frontmatter literal sets owner: None. Change to:

    owner: Some(author.clone()),

author is already in scope as the create() parameter. Because Frontmatter.owner uses #[serde(skip_serializing_if = "Option::is_none")], setting Some(...) is sufficient for it to appear in the persisted TOML frontmatter.

**2. Display owner in apm show — apm/src/cmd/show.rs ~lines 123-138**

Inside print_ticket(), add after existing metadata lines:

    if let Some(o) = &ticket.frontmatter.owner {
        println!("  owner:  {}", o);
    }

**3. Display owner in apm list — apm/src/cmd/list.rs ~lines 29-32**

Current format string:
    println!("{:<8} [{:<12}] {}", fm.id, fm.state, fm.title);

Extend to include owner (show "-" when None):
    let owner = fm.owner.as_deref().unwrap_or("-");
    println!("{:<8} [{:<12}] {:<16} {}", fm.id, fm.state, owner, fm.title);

**4. Backward compatibility — no action needed**

Existing tickets without an owner field already parse correctly: Option<String> deserialises to None when the key is absent.

**5. Tests — apm-core/tests/ticket_create.rs**

Add a test after create_returns_ticket_with_correct_fields (line 66). Use the same repo-scaffold pattern:
- Call ticket::create(...) with a known author string.
- Assert ticket.frontmatter.owner == Some(author_string).
- Re-parse the persisted markdown file and assert owner appears in the TOML frontmatter with the correct value.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T15:46Z | groomed | in_design | philippepascal |
| 2026-04-08T15:49Z | in_design | specd | claude-0408-1546-9708 |
| 2026-04-08T21:47Z | specd | ready | apm |
| 2026-04-08T21:57Z | ready | in_progress | philippepascal |
