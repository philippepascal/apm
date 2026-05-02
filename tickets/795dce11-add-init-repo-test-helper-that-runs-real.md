+++
id = "795dce11"
title = "Add init_repo() test helper that runs real apm init"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/795dce11-add-init-repo-test-helper-that-runs-real"
created_at = "2026-05-01T20:26:41.678324Z"
updated_at = "2026-05-02T03:12:32.020641Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

All current setup helpers in `apm/tests/integration.rs` (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) hand-write a minimal `apm.toml` at repo root using a hard-coded string literal and never invoke `apm init`. Because the config is synthesised offline, changes to the production init template — default workflow states, ticket section names, completion strategies, `.gitignore` entries — are invisible to the test suite. Tests pass against a fixture that diverges from what real users get.

The desired state is a single `init_repo()` helper that creates a temporary git repository by running the actual `apm init` binary, producing the same `.apm/config.toml`, `.apm/workflow.toml`, and supporting files that a real project gets. All subsequent migration tickets in this epic will compose on top of `init_repo()` rather than synthesising config from scratch.

This ticket adds only the helper and a smoke test. No existing helper is modified.

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
| 2026-05-02T03:12Z | groomed | in_design | philippepascal |