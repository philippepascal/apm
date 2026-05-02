+++
id = "464d67d5"
title = "Migrate sync test setups to init_repo()"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/464d67d5-migrate-sync-test-setups-to-init-repo"
created_at = "2026-05-01T20:27:11.656953Z"
updated_at = "2026-05-02T03:07:57.157498Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:5716 setup_sync_repo() and apm/tests/integration.rs:5894 setup_branch_in_origin() build bare-origin + clone fixtures for sync tests, with hand-rolled config in both halves. Rewrite to use init_repo() in the local clone and minimise the bare-origin scaffolding to only what cannot be done via real commands (likely a small marked bypass since you have to seed branches into a bare repo).

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
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
