+++
id = "c148f904"
title = "Migrate strict-transitions and merge-workflow setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c148f904-migrate-strict-transitions-and-merge-wor"
created_at = "2026-05-01T20:26:55.674729Z"
updated_at = "2026-05-02T03:35:03.756791Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:3747 setup_with_strict_transitions() and apm/tests/integration.rs:6850 setup_with_merge_workflow() each hand-roll workflow tables to test specific transition or completion behaviour. Rewrite both to start from init_repo() and override only the delta. Two helpers in scope, similar mechanics.

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
| 2026-05-02T03:35Z | groomed | in_design | philippepascal |
