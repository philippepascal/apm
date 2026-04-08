+++
id = "bbd5d271"
title = "Validate owner against collaborators in config-based mode"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/bbd5d271-validate-owner-against-collaborators-in-"
created_at = "2026-04-08T15:09:59.601187Z"
updated_at = "2026-04-08T15:09:59.601187Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

In config-based mode (no git_host provider), there is no validation when changing a ticket's owner. A typo in a username goes undetected. The `project.collaborators` list exists in config.toml but is never checked at runtime. Owner changes should validate the new owner against this list.

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