+++
id = "e8a56566"
title = "UI supervisor list APIs should not pull closed ticket by default"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
branch = "ticket/e8a56566-ui-supervisor-list-apis-should-not-pull-"
created_at = "2026-04-02T18:12:19.697833Z"
updated_at = "2026-04-02T18:12:19.697833Z"
+++

## Spec

### Problem

api performance is impacted and prevents regular fast refresh  from UI. Only when user select "show closed" should the UI query them along with the other tickets.

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
| 2026-04-02T18:12Z | — | new | apm-ui |
