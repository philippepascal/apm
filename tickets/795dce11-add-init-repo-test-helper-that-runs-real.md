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

- [ ] `init_repo()` compiles and is accessible to all tests in `integration.rs`
- [ ] `init_repo()` returns `TempDir` (the same type returned by existing helpers such as `setup()`)
- [ ] After `init_repo()` returns, `.apm/config.toml` exists inside the tempdir
- [ ] After `init_repo()` returns, `.apm/workflow.toml` exists inside the tempdir
- [ ] After `init_repo()` returns, a `tickets/` directory exists inside the tempdir
- [ ] After `init_repo()` returns, `.gitignore` inside the tempdir contains at least one apm-specific entry (e.g. `.apm/local.toml`)
- [ ] After `init_repo()` returns, `Config::load(dir.path())` succeeds without error
- [ ] After `init_repo()` returns, the tempdir is a valid git repository with at least one commit (HEAD resolves)
- [ ] A dedicated `#[test] fn test_init_repo_helper()` test calls `init_repo()`, asserts all of the above criteria, and passes under `cargo test`
- [ ] `init_repo()` does not emit unexpected output to stdout/stderr when run in the test harness (assertion failure on non-zero exit from `apm init` is acceptable)

### Out of scope

- Migrating any existing setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) to use `init_repo()` — each is covered by a dedicated sibling ticket in this epic
- Adding `// BYPASS:` annotations to any existing code — that work belongs to each migration ticket
- Changing the behaviour of `apm init` itself
- Any CI enforcement or linting of the bypass policy (covered by ticket 8217e5f5)
- Removing the `apm.toml` legacy fallback from `Config::load` (covered by ticket 40fdde3b, intentionally last in the epic)

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