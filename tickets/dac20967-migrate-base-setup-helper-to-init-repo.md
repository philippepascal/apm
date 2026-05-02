+++
id = "dac20967"
title = "Migrate base setup() helper to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/dac20967-migrate-base-setup-helper-to-init-repo"
created_at = "2026-05-01T20:26:43.905437Z"
updated_at = "2026-05-02T03:17:33.749832Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

apm/tests/integration.rs:34 setup() hand-writes a 6-state workflow that does not match the 12-state production default. Replace its body with a call to init_repo() (added by upstream ticket) plus any minimal overrides the tests using it actually need. Expect test fallout from tests that depended on the legacy minimal workflow — that fallout is the point: those tests are papering over real coverage gaps. Each broken test should either be updated to work with the production workflow, or be deleted if the scenario it covers no longer makes sense, with a one-line note on which is which.

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
| 2026-05-02T03:17Z | groomed | in_design | philippepascal |
