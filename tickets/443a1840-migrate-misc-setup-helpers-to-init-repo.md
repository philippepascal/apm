+++
id = "443a1840"
title = "Migrate misc setup helpers to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/443a1840-migrate-misc-setup-helpers-to-init-repo"
created_at = "2026-05-01T20:27:23.868607Z"
updated_at = "2026-05-02T04:17:07.491518Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Four setup helpers in `apm/tests/integration.rs` still hand-write config files instead of calling `apm init`:

- **`setup_with_satisfies_deps`** (line 4156): writes a legacy `apm.toml` at repo root with a 3-state workflow (`ready`, `implemented`, `closed`). Used by 3 `pick_next` tests that exercise `satisfies_deps` scheduling.
- **`setup_with_server_url`** (line 4854): calls `setup()` and appends a `[server]` block to `apm.toml`. Used by 7 auth/server tests (`register`, `sessions`, `revoke`).
- **`setup_with_archive_dir`** (line 5101): calls `setup()` and edits `apm.toml` to inject `archive_dir = "archive/tickets"`. Used by 6 archive tests.
- **`setup_on_failure_fix_project`** (line 2852): manually creates `.apm/config.toml` and a hand-crafted `.apm/workflow.toml` with 2-3 states. Used by 4 `validate --fix` tests.

All four create fixtures that diverge from what `apm init` produces: wrong config file location (legacy `apm.toml` vs `.apm/config.toml`), truncated workflow state lists, and no `.gitignore` entry. Changes to the production init template are invisible to these tests.

Each helper should be rewritten to call `init_repo()` and then apply only the one setting the tests actually exercise, using a marked `// BYPASS:` comment only where no `apm` command can make the required change.

### Acceptance criteria

- [ ] `setup_with_satisfies_deps()` body is replaced with `init_repo()` only — no `git init`, no `std::fs::write`, no `apm.toml` reference remains
- [ ] `next_skips_dep_blocked_returns_unblocked` passes
- [ ] `next_returns_dep_blocked_after_dep_satisfies` passes
- [ ] `next_picks_low_priority_blocker_before_higher_raw_independent` passes
- [ ] `setup_with_archive_dir()` body is replaced with `init_repo()` only — no `apm.toml` read/write remains
- [ ] All 6 tests calling `setup_with_archive_dir` pass
- [ ] `setup_with_server_url(url)` calls `init_repo()` and appends the `[server]` block to `.apm/config.toml` (not `apm.toml`)
- [ ] The `[server]` block injection in `setup_with_server_url` carries a `// BYPASS: no apm command configures server.url` comment
- [ ] All 7 tests calling `setup_with_server_url` pass
- [ ] `setup_on_failure_fix_project` calls `init_repo()` instead of hand-writing git init, config.toml, and workflow.toml
- [ ] Each filesystem mutation in `setup_on_failure_fix_project` (stripping `on_failure` line, removing `merge_failed` block) carries a `// BYPASS:` comment explaining why no apm command can do it
- [ ] `test_fix_adds_field_only` passes
- [ ] `test_fix_adds_state_only` passes
- [ ] `test_fix_adds_both_atomically` passes
- [ ] `test_fix_is_idempotent` passes
- [ ] No test function body is changed — only the four helper bodies are modified

### Out of scope

- Migrating `commit_ticket_to_branch()` — utility function, not a config-carrying setup helper
- Migrating `setup_epic_list()` (line 4311) or any other helper not listed in the Problem section
- Migrating `setup()`, `setup_merge()`, `setup_with_close_workflow()`, `setup_with_local_worktrees()`, `setup_with_worktrees()` — each has a dedicated sibling ticket in the epic
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Adding `apm` commands to set `server.url`, remove workflow states, or toggle `archive_dir`
- Changing any test function body (only the four helper bodies are in scope)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:17Z | groomed | in_design | philippepascal |