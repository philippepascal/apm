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
| 2026-04-08T15:09Z | — | new | philippepascal |