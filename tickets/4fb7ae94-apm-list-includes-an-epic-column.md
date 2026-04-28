+++
id = "4fb7ae94"
title = "apm list includes an epic column"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4fb7ae94-apm-list-includes-an-epic-column"
created_at = "2026-04-28T00:25:28.853946Z"
updated_at = "2026-04-28T00:57:32.160631Z"
+++

## Spec

### Problem

`apm list` currently renders four columns: ID, state, owner, and title. There is no visibility into a ticket's epic membership or base-branch context. To understand where a ticket fits in the git topology, a user must `apm show` each ticket individually.

Every ticket has an optional `target_branch` field. For epic-member tickets this holds the epic branch (e.g. `epic/8db73240-user-auth`); for standalone tickets the field is absent. When absent, the ticket's implicit base is the project's configured default branch (typically `main`).

Adding an epic/base-branch column to `apm list` exposes this topology at a glance without requiring any per-ticket drill-down.

### Acceptance criteria

- [ ] `apm list` output includes a new column between the owner column and the title column
- [ ] For a ticket whose `target_branch` frontmatter field is set, the column displays that value verbatim (e.g. `epic/8db73240-user-auth`)
- [ ] For a ticket whose `target_branch` field is absent, the column displays the project's configured default branch (e.g. `main`)
- [ ] All rows in a single `apm list` invocation use the same fixed column width so values are left-aligned in a consistent gutter
- [ ] Existing snapshot or integration tests for `apm list` pass (updated to include the new column)

### Out of scope

- Filtering `apm list` by epic or by target branch
- Resolving the epic ID to a human-readable epic title in the column
- Showing the ticket's own branch name (distinct from `target_branch`)
- Any changes to `apm show`, `apm epic list`, or other commands

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:25Z | — | new | philippepascal |
| 2026-04-28T00:26Z | new | groomed | philippepascal |
| 2026-04-28T00:57Z | groomed | in_design | philippepascal |