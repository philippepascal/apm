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

setup_with_close_workflow() at line 910 of apm/tests/integration.rs hand-writes an apm.toml at the repo root containing a 4-state workflow (new, in_progress, implemented, closed). It never calls apm init, so the fixture diverges from the production repo shape: the config is at the legacy apm.toml location instead of .apm/workflow.toml, the state list is much smaller than the production default (4 vs 12 states), and any change to the production init template — new states, field renames, config layout — is invisible to the 7 tests that depend on this helper.

The sync auto-close behavior the tests exercise depends on exactly two things from the workflow config: (1) "implemented" being a recognized non-terminal state, and (2) "closed" being a terminal state. Both are satisfied by the 12-state workflow that apm init produces, so the migration is structurally straightforward.

One test — sync_no_close_when_nothing_to_close (line 1016) — reads "apm.toml" by name to obtain a git-blob reference point. After migration this file will not exist; the path must be updated to a file that init_repo() actually writes (e.g. .apm/config.toml).

### Acceptance criteria

- [ ] setup_with_close_workflow() body is replaced with a single call to init_repo(); the hand-written apm.toml string literal is removed
- [ ] The helper no longer creates a tempdir, runs git init, sets git config, or writes any file directly
- [ ] sync_no_close_when_nothing_to_close (line 1016) is updated to reference .apm/config.toml instead of the deleted apm.toml
- [ ] All 7 tests that call setup_with_close_workflow() pass after the migration: sync_closes_implemented_ticket_on_merged_branch, sync_closes_implemented_ticket_with_no_branch, sync_no_close_when_nothing_to_close, sync_closes_multiple_tickets_on_merged_branches, sync_handler_closes_merged_ticket, sync_handler_no_close_returns_zero, sync_closes_implemented_ticket_with_merged_branch_in_one_run
- [ ] Any deviation from production behavior that cannot be replicated via a real apm command is annotated with // BYPASS:
- [ ] cargo test passes with no regressions in the sync test group

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