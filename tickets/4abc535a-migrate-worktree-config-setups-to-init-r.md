+++
id = "4abc535a"
title = "Migrate worktree-config setups to init_repo()"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4abc535a-migrate-worktree-config-setups-to-init-r"
created_at = "2026-05-01T20:27:01.767841Z"
updated_at = "2026-05-02T03:49:22.862715Z"
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

Both helpers are in apm/tests/integration.rs. Changes are isolated to those two function bodies; no callers are modified.

**setup_with_worktrees() (line 3578) -- no workers, simpler**

Replace the entire function body with:

1. Call init_repo() and bind its return: let dir = init_repo(); let p = dir.path();
2. Return dir.

No config override is required. apm init already writes [worktrees] dir = "worktrees" inside the repo, which is inside the tempdir, achieving the same isolation the hand-written config provided. The production workflow has all states the three worker-kill tests need (they only exercise the git-worktree and PID-file machinery, not state transitions).

**setup_with_local_worktrees() (line 1725) -- has workers config**

1. Call init_repo(): let dir = init_repo(); let p = dir.path();
2. Create the mock worker (order unchanged): let mock_worker = make_mock_worker(p);
3. BYPASS: append the workers section to .apm/config.toml -- no CLI command exists for this field:
   - Read the existing config: std::fs::read_to_string(p.join(".apm/config.toml"))
   - Append the [workers] table: push_str with a newline + [workers] + newline + command = "<mock_worker_path>" + newline
   - Write the file back with std::fs::write
   - Annotate the block with // BYPASS: no CLI command to set workers.command post-init
4. Commit the config change so HEAD stays valid for worktree operations:
   git(p, &["add", ".apm/config.toml"]);
   git(p, &["commit", "-m", "add workers config"]);
5. Return dir.

The production workflow provides new, ready (actionable, with command:start transition to in_progress), in_progress, closed -- all states exercised by the 15 callers. No state-level override is needed.

**File**

Only apm/tests/integration.rs changes. No other files are touched.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:44Z | groomed | in_design | philippepascal |