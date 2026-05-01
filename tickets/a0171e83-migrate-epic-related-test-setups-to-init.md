+++
id = "a0171e83"
title = "Migrate epic-related test setups to init_repo() + real apm epic"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a0171e83-migrate-epic-related-test-setups-to-init"
created_at = "2026-05-01T20:27:07.814641Z"
updated_at = "2026-05-01T20:29:10.917064Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs has four epic-related setups: setup_with_epic (line 2535), setup_with_epic_for_owner_tests (5465), setup_epic_list (4311), setup_epic_show (4431). All hand-roll the configuration and synthesize epic state directly. Rewrite each to use init_repo() for config and `apm epic new` for epic creation; mark any unavoidable bypass with `// BYPASS:`. Four helpers in scope, all using the same epic primitives.

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