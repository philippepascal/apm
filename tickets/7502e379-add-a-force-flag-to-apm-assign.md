+++
id = "7502e379"
title = "add a force flag to apm assign"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7502e379-add-a-force-flag-to-apm-assign"
created_at = "2026-04-08T23:57:24.004823Z"
updated_at = "2026-04-09T00:10:24.252918Z"
+++

## Spec

### Problem

The `apm assign` command currently only allows the current ticket owner to reassign ownership. Any attempt by a non-owner to run `apm assign <id> <user>` fails with "only the current owner (<owner>) can reassign this ticket". There is no escape hatch for administrators or collaborators who need to take over or hand off a ticket when the current owner is unavailable.

A `--force` flag would let any collaborator override the ownership check, while a confirmation prompt prevents accidental overrides by requiring explicit acknowledgement of the current owner before proceeding.

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
| 2026-04-08T23:57Z | — | new | philippepascal |
| 2026-04-08T23:57Z | new | groomed | apm |
| 2026-04-09T00:10Z | groomed | in_design | philippepascal |