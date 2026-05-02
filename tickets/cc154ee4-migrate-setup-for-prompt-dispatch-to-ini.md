+++
id = "cc154ee4"
title = "Migrate setup_for_prompt_dispatch() to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/cc154ee4-migrate-setup-for-prompt-dispatch-to-ini"
created_at = "2026-05-01T20:27:03.975333Z"
updated_at = "2026-05-02T03:49:50.299843Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_for_prompt_dispatch()` at `apm/tests/integration.rs:2099` hand-rolls a 6-state workflow (`new`, `in_design`, `ammend`, `ready`, `in_progress`, `closed`) with `trigger = "command:start"` transitions on `new`, `ammend`, and `ready`. It writes the config to the legacy `apm.toml` root location, never calls `apm init`, and creates the `.apm/` directory manually.

This diverges from the production repo shape in two ways. First, the config file is at the wrong location (`apm.toml` instead of `.apm/config.toml`). Second, the `new` state having `command:start → in_design` is a custom invention — in production, the dispatch path for spec-writing is `groomed → in_design`. Tests `spawn_new_ticket_transitions_to_in_design` and `start_next_spawn_new_ticket_transitions_correctly` therefore exercise dispatch against a non-production state, masking any breakage in the real `groomed` dispatch path.

There are 7 tests that depend on this helper. They cover owner-preservation semantics on `in_design` transitions, and the prompt-dispatch mechanism for `ammend → in_design`, `ready → in_progress`, and the `groomed → in_design` path (currently exercised via the ersatz `new` state).

### Acceptance criteria

- [ ] `setup_for_prompt_dispatch()` no longer writes `apm.toml`; it calls `init_repo()` as its first step
- [ ] The test repo produced by `setup_for_prompt_dispatch()` has `.apm/config.toml` (not `apm.toml`) as its config file
- [ ] The mock worker path is injected into `.apm/config.toml` so that `apm start --spawn` can invoke it
- [ ] The injection is marked with a `// BYPASS:` comment explaining why direct file editing is used
- [ ] `spawn_new_ticket_transitions_to_in_design` passes using a `groomed`-state ticket (matching the production dispatch path)
- [ ] `start_next_spawn_new_ticket_transitions_correctly` passes using a `groomed`-state ticket
- [ ] `spawn_ammend_ticket_transitions_to_in_design` passes unchanged (production workflow already has `ammend → in_design` via `command:start`)
- [ ] `spawn_ready_ticket_transitions_to_in_progress` passes unchanged (production workflow already has `ready → in_progress` via `command:start`)
- [ ] `start_next_spawn_ready_ticket_transitions_correctly` passes unchanged
- [ ] `in_design_does_not_set_owner_when_unowned` passes unchanged
- [ ] `in_design_does_not_overwrite_different_owner` passes unchanged
- [ ] All 7 tests pass under `cargo test` with no modifications to test assertions

### Out of scope

- Migrating any other setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in this epic
- Replacing `write_ticket_to_branch()` / `write_ticket_with_owner()` direct file writes with real `apm new` + `apm state` calls — covered by sibling ticket 059e2e74
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Adding a CLI command to configure `workers.command` post-init — that is a product feature decision
- Changing any test assertion or the behavior being tested — only the fixture setup is in scope

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:49Z | groomed | in_design | philippepascal |