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

The `setup()` helper at `apm/tests/integration.rs:34` hand-writes a 6-state `apm.toml` at repo root using a hard-coded string literal. Its workflow contains: `new`, `specd`, `ammend`, `ready`, `in_progress`, `closed`. The production default — produced by `apm init` — contains 12 states: `new`, `groomed`, `question`, `specd`, `ammend`, `in_design`, `ready`, `in_progress`, `blocked`, `implemented`, `merge_failed`, `closed`.

Because the test fixture diverges from production, two categories of problem arise. First, changes to the production init template (new states, changed transitions, new config keys) are completely invisible to the ~122 tests that use `setup()`. Second, the 6-state workflow has transition rules that do not exist in production: `new → specd`, `new → ready`, and `new → in_progress` are all reachable in the test fixture but are not valid transitions in production. Tests that exercise those paths are silently asserting behaviour that real users cannot trigger.

The fix is to replace `setup()` body with a call to `init_repo()` (added by upstream ticket 795dce11). This will surface test failures for any test that relied on the non-production transition rules or the reduced state set. Those failures are not noise — they represent real coverage gaps. Each broken test must be triaged: updated to work with production workflow semantics, or deleted if the scenario it covers no longer makes sense, with a short inline note explaining the decision.

### Acceptance criteria

- [ ] `setup()` body no longer contains any `std::fs::write` call for `apm.toml` or any hand-written config string
- [ ] `setup()` delegates entirely to `init_repo()` and returns its `TempDir` directly
- [ ] `setup()` still returns a `TempDir` so all call sites continue to use `dir.path()` without modification
- [ ] `cargo test --test integration` exits 0 after the migration (all tests either pass or are explicitly removed)
- [ ] Every test that was updated to accommodate production workflow semantics compiles and passes with its new transition path
- [ ] Every test that was deleted has a `// DELETED: <one-line reason>` comment in its former location committed to history
- [ ] No test calls `apm::cmd::state::run` with `force: false` on a transition that is not valid in the production workflow (e.g. `new → specd` without force)
- [ ] The repo root no longer contains `apm.toml` after `setup()` runs (the production config lives at `.apm/config.toml`)

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