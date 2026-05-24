+++
id = "c8bd9b55"
title = "Use mock-happy/mock-sad in integration tests instead of debug wrapper"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8bd9b55-use-mock-happy-mock-sad-in-integration-t"
created_at = "2026-05-24T19:07:11.167447Z"
updated_at = "2026-05-24T19:07:11.167447Z"
+++

## Spec

### Problem

What is broken or missing, and why it matters.

### Acceptance criteria

Checkboxes; each one independently testable.
- [ ] Replace "debug/" with "mock-happy/" in workflow.toml patch in both setup functions
- [ ] Remove make_mock_worker (dead code)
- [ ] Remove APM_SKIP_COMPAT_CHECK from release.yml
- [ ] Spawn tests call apm_core::start::run directly, wait for child process, and assert final ticket state after mock-happy completes

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
| 2026-05-24T19:07Z | — | new | philippepascal |