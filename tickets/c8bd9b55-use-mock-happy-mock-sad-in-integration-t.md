+++
id = "c8bd9b55"
title = "Use mock-happy/mock-sad in integration tests instead of debug wrapper"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8bd9b55-use-mock-happy-mock-sad-in-integration-t"
created_at = "2026-05-24T19:07:11.167447Z"
updated_at = "2026-05-24T19:34:16.805155Z"
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

Adding new tests beyond the spawn tests already present. Adding mock-sad or mock-random coverage (separate ticket if wanted).

### Approach

In setup_with_local_worktrees and setup_for_prompt_dispatch, change the workflow.toml patch from `"debug/` to `"mock-happy/`. Then refactor the spawn tests to call `apm_core::start::run` directly (instead of the CLI wrapper) so they get the `StartOutput` back, extract the child process handle, wait for it to exit, and then assert the final ticket state. Remove `make_mock_worker` and `APM_SKIP_COMPAT_CHECK` once nothing needs them.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T19:07Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
