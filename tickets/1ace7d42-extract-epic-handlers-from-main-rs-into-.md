+++
id = "1ace7d42"
title = "Extract epic handlers from main.rs into handlers/epics.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1ace7d42-extract-epic-handlers-from-main-rs-into-"
created_at = "2026-04-12T09:03:14.832182Z"
updated_at = "2026-04-12T09:49:07.015419Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["7bb8eacb"]
+++

## Spec

### Problem

`apm-server/src/main.rs` is a ~4 000-line file that mixes HTTP handler logic, data structures, server bootstrap, and worker-queue code. Epic-related handler code — roughly 220 lines of production code plus ~185 lines of tests — is defined inline in main.rs alongside unrelated concerns.

The desired state mirrors the pattern established by the sibling ticket for ticket handlers (7bb8eacb): all epic HTTP handler code lives in `apm-server/src/handlers/epics.rs`, and `main.rs` retains only route registrations that reference the moved functions by path. After both extractions main.rs shrinks by ~700 lines total and is navigable by concern.

Note on `parse_epic_branch`: this function duplicates the slug-to-title logic in `apm/src/cmd/epic.rs`. Moving it to `handlers/epics.rs` as-is is correct for this ticket. Replacing it with a shared `apm_core::epic` helper is explicitly out of scope and belongs to a separate refactor.

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
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:49Z | groomed | in_design | philippepascal |