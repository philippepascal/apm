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

Integration tests that exercise worker spawning use the `debug` wrapper, which is a no-op (exits immediately without doing anything). This means the tests only verify that the spawn path doesn't error — they don't verify that a real agent loop (claim ticket → run → transition state → exit) works end-to-end. The mock agents (`mock-happy`, `mock-sad`) were built exactly for this purpose but are unused in the test suite. Additionally there is leftover dead code (`make_mock_worker`) and unnecessary CI plumbing (`APM_SKIP_COMPAT_CHECK`) from earlier failed attempts to make CI work without a real claude binary.

### Acceptance criteria

Checkboxes; each one independently testable.
- [ ] Replace "debug/" with "mock-happy/" in workflow.toml patch in both setup functions
- [ ] Remove make_mock_worker (dead code)
- [ ] Remove APM_SKIP_COMPAT_CHECK from release.yml
- [ ] Spawn tests call apm_core::start::run directly, wait for child process, and assert final ticket state after mock-happy completes
- [ ] All 259 integration tests pass without APM_SKIP_COMPAT_CHECK set

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