+++
id = "296c1061"
title = "Migrate setup_with_close_workflow() to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/296c1061-migrate-setup-with-close-workflow-to-ini"
created_at = "2026-05-01T20:26:48.501162Z"
updated_at = "2026-05-02T03:28:32.522775Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:910 setup_with_close_workflow() hand-rolls a workflow that includes implemented→closed for sync tests. Rewrite to compose on init_repo() output. Expected test fallout — fix or delete on a per-test basis with notes.

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
| 2026-05-02T03:28Z | groomed | in_design | philippepascal |
