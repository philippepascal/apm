+++
id = "4abc535a"
title = "Migrate worktree-config setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4abc535a-migrate-worktree-config-setups-to-init-r"
created_at = "2026-05-01T20:27:01.767841Z"
updated_at = "2026-05-02T03:44:18.219029Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

setup_with_local_worktrees() (line 1725) and setup_with_worktrees() (line 3578) in apm/tests/integration.rs both hand-write a full apm.toml at the repo root. Neither calls apm init. As a result, the fixtures diverge from the production repo shape in two ways: the config file is at the legacy apm.toml location instead of .apm/config.toml, and the hand-crafted workflow states are a smaller, frozen subset of the production default.

The [worktrees] dir = worktrees present in both helpers is included to keep worktrees inside the tempdir during tests. That value happens to be identical to what apm init now writes by default, so no override of this field is needed after migration -- init_repo() already provides it.

setup_with_local_worktrees() additionally sets [workers] command to a mock binary. There is no CLI command that configures workers.command post-init, so this one field must be injected via a direct edit to .apm/config.toml with a // BYPASS: annotation per the epic bypass policy.

setup_with_local_worktrees() is called by 15 tests (start / work commands). setup_with_worktrees() is called by 3 tests (workers kill-process commands).

### Acceptance criteria

- [ ] setup_with_worktrees() calls init_repo() instead of manually calling git init and writing apm.toml
- [ ] setup_with_local_worktrees() calls init_repo() instead of manually calling git init and writing apm.toml
- [ ] Neither helper writes apm.toml at the repo root; config lives at .apm/config.toml as produced by apm init
- [ ] Neither helper performs its own git init or initial commit
- [ ] setup_with_local_worktrees() adds [workers] command to .apm/config.toml via a direct file edit annotated with // BYPASS: no CLI command to set workers.command post-init
- [ ] The workers config edit is committed to git before the helper returns
- [ ] No [worktrees] dir override is written by either helper (the apm init default dir = worktrees already satisfies test isolation)
- [ ] make_mock_worker() is still called and its path is used in the workers.command bypass write
- [ ] All 15 callers of setup_with_local_worktrees() continue to pass without modification
- [ ] All 3 callers of setup_with_worktrees() continue to pass without modification

### Out of scope

- Migrating any other setup helper (setup(), setup_merge(), setup_aggressive(), etc.) -- each has its own sibling ticket in the epic
- Adding a CLI command to configure workers.command or other config fields post-init
- Removing the apm.toml legacy fallback from Config::load -- covered by ticket 40fdde3b, intentionally last in the epic
- Changing the behaviour of any test that calls these helpers
- Migrating setup_merge_strategy_remote(), setup_squash_remote(), or any remote/clone fixture -- covered by sibling ticket 094838b6
- The init_repo() implementation itself -- covered by ticket 795dce11

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
| 2026-05-02T03:44Z | groomed | in_design | philippepascal |