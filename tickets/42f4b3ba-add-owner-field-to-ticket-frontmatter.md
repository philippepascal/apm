+++
id = "42f4b3ba"
title = "Add owner field to ticket frontmatter"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/42f4b3ba-add-owner-field-to-ticket-frontmatter"
created_at = "2026-04-04T06:28:01.284791Z"
updated_at = "2026-04-04T06:28:01.284791Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

The ticket frontmatter has `author` (who created it) and `supervisor` (who reviews it) but no field to track who is currently working on it. The UI has an "agent" filter dropdown that renders but does nothing because there is no corresponding field in the Frontmatter struct or API response. Without an ownership field, there is no way to answer "which tickets is Alice currently responsible for?" — you can only see who created them.

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