+++
id = "094838b6"
title = "Migrate remote-merge test setups to init_repo()"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/094838b6-migrate-remote-merge-test-setups-to-init"
created_at = "2026-05-01T20:27:20.736073Z"
updated_at = "2026-05-01T20:27:20.736073Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm/tests/integration.rs has three bare-origin + clone variants for merge-strategy testing: setup_squash_remote (3914), setup_pr_or_epic_merge_remote (4710), setup_merge_strategy_remote (5306). All hand-roll completion strategy configs. Rewrite each to use init_repo() and override the [completion] section via real commands or marked bypass. Three helpers, similar mechanics.

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
