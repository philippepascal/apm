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

- [ ] `apm assign --force <id> <user>` succeeds when the current user is not the ticket owner
- [ ] When `--force` is used and the ticket has an existing owner, a prompt shows "Ticket <id> is currently owned by <owner>. Reassign to <user>? [y/N]" before proceeding
- [ ] Entering `y` or `Y` at the prompt completes the assignment
- [ ] Entering anything other than `y`/`Y` (including empty input) aborts with message "aborted" and leaves the ticket unchanged
- [ ] `--force` on an unowned ticket proceeds without showing a confirmation prompt
- [ ] `--force` does not bypass the terminal-state guard — `apm assign --force <id> <user>` on a closed ticket still errors with "cannot change owner of a closed ticket"
- [ ] `--force` still validates the target username against the configured collaborators list
- [ ] Without `--force`, the existing behaviour is unchanged: a non-owner gets the error "only the current owner (<owner>) can reassign this ticket"

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