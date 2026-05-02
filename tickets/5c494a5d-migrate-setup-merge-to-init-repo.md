+++
id = "5c494a5d"
title = "Migrate setup_merge() to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5c494a5d-migrate-setup-merge-to-init-repo"
created_at = "2026-05-01T20:26:46.198163Z"
updated_at = "2026-05-02T03:22:43.659646Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:134 setup_merge() hand-writes a workflow that adds in_progress→implemented to enable depends_on tests. Rewrite to call init_repo() then apply only the delta (the depends_on/implemented hookup) via real apm commands or, where unavoidable, a marked `// BYPASS:` filesystem edit. All tests using setup_merge must pass against the production workflow shape.

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
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:22Z | groomed | in_design | philippepascal |
