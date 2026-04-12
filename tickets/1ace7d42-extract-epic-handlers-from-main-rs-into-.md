+++
id = "1ace7d42"
title = "Extract epic handlers from main.rs into handlers/epics.rs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1ace7d42-extract-epic-handlers-from-main-rs-into-"
created_at = "2026-04-12T09:03:14.832182Z"
updated_at = "2026-04-12T09:03:14.832182Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["7bb8eacb"]
+++

## Spec

### Problem

`apm-server/src/main.rs` contains ~300 lines of epic handler functions that should be in their own module. These include:

- `list_epics()` — lists all epic branches with derived state
- `get_epic()` — loads epic details and associated tickets
- `create_epic()` — creates new epic branch
- `parse_epic_branch()` — extracts ID and title from branch name (utility helper)
- Epic-related serialization logic

These handlers also contain inline `branch_to_title`-style logic (parsing epic branch names into display titles) which duplicates what `apm/src/cmd/epic.rs` does. After the apm CLI refactoring epic moves `branch_to_title` and `epic_id_from_branch` into `apm_core::epic`, these handlers should use the shared helpers.

Extracting into `handlers/epics.rs` will reduce main.rs by ~300 lines. This ticket depends on the ticket handlers being extracted first to avoid merge conflicts (both modify main.rs).

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