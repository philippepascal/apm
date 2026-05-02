+++
id = "296c1061"
title = "Migrate setup_with_close_workflow() to init_repo()"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/296c1061-migrate-setup-with-close-workflow-to-ini"
created_at = "2026-05-01T20:26:48.501162Z"
updated_at = "2026-05-02T03:34:41.438444Z"
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

- Migrating any other helper (setup(), setup_merge(), setup_aggressive(), etc.) — each is covered by a dedicated sibling ticket in this epic
- Replacing write_ticket_to_branch() direct file writes with apm new + state commands — covered by sibling ticket 059e2e74
- Removing the apm.toml legacy fallback from Config::load — covered by ticket 40fdde3b, intentionally last in the epic
- Adding new apm commands to configure individual workflow states or transitions
- Changing any apm sync behavior or the sync auto-close detection logic itself

### Approach

File: apm/tests/integration.rs

**Step 1 — Replace the helper body (around line 910)**

Delete the entire body of setup_with_close_workflow() and replace with:

    fn setup_with_close_workflow() -> TempDir {
        init_repo()
    }

No workflow overrides needed. The production 12-state workflow produced by apm init already includes both "implemented" (non-terminal) and "closed" (terminal), which is all the sync tests require. The git user config calls (git config user.email / user.name) and manual create_dir_all for tickets/ are also unnecessary — init_repo() handles them via the git() helper and apm init.

**Step 2 — Fix sync_no_close_when_nothing_to_close (line ~1016)**

This test calls branch_content(p, "main", "apm.toml") solely to hold a reference to a file on main. After migration, apm.toml does not exist. Change the filename argument from "apm.toml" to ".apm/config.toml":

    // Before
    let log_before = branch_content(p, "main", "apm.toml");
    // After
    let log_before = branch_content(p, "main", ".apm/config.toml");

The variable is used only via drop(log_before) at the end; its value is irrelevant.

**Step 3 — No changes to the other 6 tests**

All remaining tests use write_ticket_to_branch() to inject tickets with state = "implemented" directly via file write. The sync logic matches on the state name string, not workflow position, so the expanded state list in the production workflow does not affect them. These direct writes are already a known bypass pattern; annotating them with // BYPASS: is out of scope for this ticket (covered by 059e2e74).

**Step 4 — Verify**

Run: cargo test sync_

All 7 tests in the sync group should pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:28Z | groomed | in_design | philippepascal |