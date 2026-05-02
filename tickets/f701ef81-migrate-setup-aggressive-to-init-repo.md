+++
id = "f701ef81"
title = "Migrate setup_aggressive() to init_repo()"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f701ef81-migrate-setup-aggressive-to-init-repo"
created_at = "2026-05-01T20:26:58.392091Z"
updated_at = "2026-05-02T03:07:44.809305Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:1584 setup_aggressive() hand-writes a config with [sync] aggressive = true. Rewrite to call init_repo() then flip the single sync.aggressive field via the proper command (or marked bypass). The sync.aggressive default may differ from what existing tests assume — note any test divergence.

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
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
