+++
id = "c8bd9b55"
title = "Use mock-happy/mock-sad in integration tests instead of debug wrapper"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8bd9b55-use-mock-happy-mock-sad-in-integration-t"
created_at = "2026-05-24T19:07:11.167447Z"
updated_at = "2026-05-24T19:34:29.132806Z"
+++

## Spec

### Problem

Integration tests that exercise worker spawning configure the `debug/worker` profile, which is a built-in no-op: it exits immediately without reading the ticket, writing spec sections, or calling `apm state`. Tests that use it only verify that `apm start --spawn` does not return an error and that the parent-side state transition (e.g. `groomed → in_design`, `ready → in_progress`) was written to the ticket branch. They say nothing about whether the agent loop itself — claim ticket, do work, call `apm state`, exit — functions end-to-end.

Two mock agents, `mock-happy` and `mock-sad`, were created specifically to fill this gap. `mock-happy` writes dummy spec or implementation content and calls `apm state <id> <success-target>` before exiting; `mock-sad` calls `apm state <id> <non-success-target>`. Neither is used in the integration tests. Additionally, a helper `make_mock_worker` exists in the test file as dead code (never called), and `APM_SKIP_COMPAT_CHECK=1` is set in CI to suppress a compat check that is irrelevant once `debug/` is replaced by a named built-in wrapper.

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
| 2026-05-24T19:34Z | groomed | in_design | philippepascal |