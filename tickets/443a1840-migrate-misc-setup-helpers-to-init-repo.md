+++
id = "443a1840"
title = "Migrate misc setup helpers to init_repo()"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/443a1840-migrate-misc-setup-helpers-to-init-repo"
created_at = "2026-05-01T20:27:23.868607Z"
updated_at = "2026-05-01T20:27:23.868607Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm/tests/integration.rs has four leftover setup helpers: setup_with_server_url (4859), setup_with_archive_dir (5106), setup_with_satisfies_deps (4156), setup_on_failure_fix_project (2852). Each tests one orthogonal config feature. Rewrite each to use init_repo() and override only the relevant feature via real commands or marked bypass. Four helpers, low coupling.

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
