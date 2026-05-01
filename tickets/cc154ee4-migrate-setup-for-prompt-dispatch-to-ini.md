+++
id = "cc154ee4"
title = "Migrate setup_for_prompt_dispatch() to init_repo()"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/cc154ee4-migrate-setup-for-prompt-dispatch-to-ini"
created_at = "2026-05-01T20:27:03.975333Z"
updated_at = "2026-05-01T20:27:03.975333Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm/tests/integration.rs:2099 setup_for_prompt_dispatch() hand-rolls a workflow with start/dispatch transitions for prompt-dispatch tests. Rewrite to start from init_repo() and add only the prompt-dispatch deltas via real commands.

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
