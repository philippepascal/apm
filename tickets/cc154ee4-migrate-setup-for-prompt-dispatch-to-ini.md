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
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:49Z | groomed | in_design | philippepascal |