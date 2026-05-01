+++
id = "296c1061"
title = "Migrate setup_with_close_workflow() to init_repo()"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/296c1061-migrate-setup-with-close-workflow-to-ini"
created_at = "2026-05-01T20:26:48.501162Z"
updated_at = "2026-05-01T20:26:48.501162Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
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
