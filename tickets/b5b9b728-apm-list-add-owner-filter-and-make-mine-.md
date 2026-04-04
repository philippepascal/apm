+++
id = "b5b9b728"
title = "apm list: add --owner filter and make --mine match author or owner"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/b5b9b728-apm-list-add-owner-filter-and-make-mine-"
created_at = "2026-04-04T06:28:11.099983Z"
updated_at = "2026-04-04T06:35:11.259944Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["42f4b3ba"]
+++

## Spec

### Problem

`apm list --mine` currently matches only the `author` field. If I started working on a ticket someone else created, `--mine` won't show it. There is also no `--owner` flag to filter by who is currently working on a ticket. The mental model of "my tickets" should include both tickets I created and tickets I'm currently responsible for.

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
| 2026-04-04T06:28Z | — | new | apm |
| 2026-04-04T06:35Z | new | groomed | apm |
