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
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:17Z | groomed | in_design | philippepascal |