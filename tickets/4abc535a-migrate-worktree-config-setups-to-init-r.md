+++
id = "4abc535a"
title = "Migrate worktree-config setups to init_repo()"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4abc535a-migrate-worktree-config-setups-to-init-r"
created_at = "2026-05-01T20:27:01.767841Z"
updated_at = "2026-05-01T20:29:03.840748Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:1725 setup_with_local_worktrees() and apm/tests/integration.rs:3578 setup_with_worktrees() override [worktrees] dir to keep parallel tests isolated inside the tempdir. Production now writes a worktrees.dir default during apm init; rewrite both helpers to compose on init_repo() and override only the dir setting.

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