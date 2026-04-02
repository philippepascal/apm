+++
id = "48105624"
title = "apm-server: embed apm-ui static assets at build time via include_dir"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/48105624-apm-server-embed-apm-ui-static-assets-at"
created_at = "2026-04-02T20:54:40.869103Z"
updated_at = "2026-04-02T20:54:40.869103Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
+++

## Spec

### Problem

apm-server requires a separately running Vite dev server to serve the UI. For distribution as a single binary, the built UI static assets must be embedded in the server at compile time using `include_dir!` or equivalent. Without this, deploying apm-server requires a separate static file deployment step. See `initial_specs/DESIGN-users.md` point 6.

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
| 2026-04-02T20:54Z | — | new | apm |