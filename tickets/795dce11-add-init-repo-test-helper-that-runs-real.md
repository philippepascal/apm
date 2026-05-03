+++
id = "795dce11"
title = "Add init_repo() test helper that runs real apm init"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/795dce11-add-init-repo-test-helper-that-runs-real"
created_at = "2026-05-01T20:26:41.678324Z"
updated_at = "2026-05-03T20:34:21.501119Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

All current setup helpers in `apm/tests/integration.rs` (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) hand-write a minimal `apm.toml` at repo root using a hard-coded string literal and never invoke `apm init`. Because the config is synthesised offline, changes to the production init template — default workflow states, ticket section names, completion strategies, `.gitignore` entries — are invisible to the test suite. Tests pass against a fixture that diverges from what real users get.

The desired state is a single `init_repo()` helper that creates a temporary git repository by running the actual `apm init` binary, producing the same `.apm/config.toml`, `.apm/workflow.toml`, and supporting files that a real project gets. All subsequent migration tickets in this epic will compose on top of `init_repo()` rather than synthesising config from scratch.

This ticket adds only the helper and a smoke test. No existing helper is modified.

### Acceptance criteria

- [x] `init_repo()` compiles and is accessible to all tests in `integration.rs`
- [x] `init_repo()` returns `TempDir` (the same type returned by existing helpers such as `setup()`)
- [x] After `init_repo()` returns, `.apm/config.toml` exists inside the tempdir
- [x] After `init_repo()` returns, `.apm/workflow.toml` exists inside the tempdir
- [x] After `init_repo()` returns, a `tickets/` directory exists inside the tempdir
- [x] After `init_repo()` returns, `.gitignore` inside the tempdir contains at least one apm-specific entry (e.g. `.apm/local.toml`)
- [x] After `init_repo()` returns, `Config::load(dir.path())` succeeds without error
- [x] After `init_repo()` returns, the tempdir is a valid git repository with at least one commit (HEAD resolves)
- [x] A dedicated `#[test] fn test_init_repo_helper()` test calls `init_repo()`, asserts all of the above criteria, and passes under `cargo test`
- [x] `init_repo()` does not emit unexpected output to stdout/stderr when run in the test harness (assertion failure on non-zero exit from `apm init` is acceptable)

### Out of scope

- Migrating any existing setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) to use `init_repo()` — each is covered by a dedicated sibling ticket in this epic
- Adding `// BYPASS:` annotations to any existing code — that work belongs to each migration ticket
- Changing the behaviour of `apm init` itself
- Any CI enforcement or linting of the bypass policy (covered by ticket 8217e5f5)
- Removing the `apm.toml` legacy fallback from `Config::load` (covered by ticket 40fdde3b, intentionally last in the epic)

### Approach

Add `init_repo()` near the top of `apm/tests/integration.rs`, alongside the existing `setup()` and `git()` helpers (roughly line 34 area).

**Function signature**

```rust
fn init_repo() -> TempDir
```

Returns `TempDir` so callers hold the directory alive via RAII, identical to the existing `setup()` convention. Callers access the path via `.path()`.

**Function body — ordered steps**

1. Create tempdir: `let dir = tempfile::tempdir().unwrap();`
2. Run `git init -q -b main` via the existing `git()` helper (it already injects `GIT_AUTHOR_*` / `GIT_COMMITTER_*` env vars so commits work without a global git config)
3. Invoke the real `apm init` binary:
   ```rust
   let bin = env!("CARGO_BIN_EXE_apm");
   let out = std::process::Command::new(bin)
       .args(["init", "--no-claude", "--quiet"])
       .current_dir(dir.path())
       .output()
       .unwrap();
   assert!(out.status.success(), "apm init failed: {}", String::from_utf8_lossy(&out.stderr));
   ```
   - `--no-claude` skips writing `.claude/settings.json` (irrelevant in tempdir and avoids touching real user config)
   - `--quiet` suppresses informational output; the assert provides a clear failure message if init exits non-zero
   - stdin is not a TTY in the test harness, so `apm init` skips all interactive prompts automatically
4. Commit the generated files so HEAD resolves (required for worktree and branch operations used by sibling tests):
   ```rust
   git(dir.path(), &["add", "."]);
   git(dir.path(), &["commit", "-m", "init"]);
   ```
5. Return `dir`

**Smoke test**

Add `#[test] fn test_init_repo_helper()` immediately after the helper. It should:
- Call `init_repo()`, bind to `let dir = init_repo(); let p = dir.path();`
- Assert `.apm/config.toml` exists: `assert!(p.join(".apm/config.toml").exists())`
- Assert `.apm/workflow.toml` exists
- Assert `tickets/` dir exists
- Assert `.gitignore` contains `".apm/local.toml"` (a known apm-injected entry)
- Assert `Config::load(p).is_ok()`
- Assert HEAD resolves: run `git(p, &["rev-parse", "HEAD"])` succeeds (or use `std::process::Command` with a status check)

**Placement in file**

Insert after the `git()` helper and before the `setup()` helper so it is visible to all callers without a forward-reference issue. No existing function is removed or changed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:12Z | groomed | in_design | philippepascal |
| 2026-05-02T03:17Z | in_design | specd | claude-0502-0312-5c20 |
| 2026-05-03T20:16Z | specd | ready | philippepascal |
| 2026-05-03T20:34Z | ready | in_progress | philippepascal |