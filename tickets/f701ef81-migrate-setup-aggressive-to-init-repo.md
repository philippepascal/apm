+++
id = "f701ef81"
title = "Migrate setup_aggressive() to init_repo()"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f701ef81-migrate-setup-aggressive-to-init-repo"
created_at = "2026-05-01T20:26:58.392091Z"
updated_at = "2026-05-03T20:16:57.049866Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_aggressive()` at line 1584 of `apm/tests/integration.rs` hand-writes a minimal `apm.toml` at the repo root with a 5-state workflow and `[sync] aggressive = true`. Like all the hand-written helpers in this epic, it never calls `apm init`, so its fixture diverges from what real users get: the config lives at the legacy root-level path (`apm.toml`) instead of `.apm/config.toml`, and the workflow has fewer states than the production default.

Six tests depend on this helper. They test two behaviours: (a) when `aggressive = true`, commands that attempt a remote fetch do not abort when no remote is configured; and (b) when the caller passes `--no-aggressive`, the fetch is suppressed entirely. Neither behaviour is tied to specific workflow states — the tests exercise `apm new`, `apm next`, `apm list`, `apm close`, `apm spec`, and `apm set`.

Crucially, `sync.aggressive` already defaults to `true` in production: `SyncConfig` carries `#[serde(default = "default_true")]` on the field, and its `Default` impl sets `aggressive: true`. The `apm init` template does not write a `[sync]` section at all, so every repo created by `init_repo()` inherits the default — aggressive mode is on without any explicit config entry. This means the migration requires no bypass: replacing the helper body with `init_repo()` is sufficient.

### Acceptance criteria

- [ ] `setup_aggressive()` no longer hand-writes `apm.toml`, calls `git init` directly, or configures `git user.email` / `git user.name`
- [ ] `setup_aggressive()` delegates repo creation entirely to `init_repo()`
- [ ] All 6 tests that call `setup_aggressive()` pass without modification to the test bodies
- [ ] No `// BYPASS:` annotation is required (the production default for `sync.aggressive` is already `true`)
- [ ] `cargo test --test integration -- aggressive` passes with the migrated helper

### Out of scope

- Migrating any other setup helper — each has its own ticket in this epic
- Changing any of the 6 test function bodies beyond what the helper change requires
- Adding an `apm config set` command to modify `sync.aggressive` at runtime — that is a product feature decision
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Migrating `setup_sync_repo()` or `setup_branch_in_origin()` — covered by ticket 464d67d5
- Changing the behaviour of `apm init` itself

### Approach

**File:** `apm/tests/integration.rs`, `setup_aggressive()` at line ~1584.

Replace the entire function body with a single delegation to `init_repo()`:

```rust
fn setup_aggressive() -> TempDir {
    init_repo()
}
```

**Why no bypass is needed:** `SyncConfig::aggressive` carries `#[serde(default = "default_true")]` and its `Default` impl returns `true`. The `apm init` template writes no `[sync]` section, so TOML deserialization falls back to the serde default — aggressive mode is on in every `init_repo()` repo without an explicit config entry.

**Why removing the extra git config calls is safe:** The hand-written body calls `git(p, &["config", "user.email", "..."])` and `git(p, &["config", "user.name", "..."])` after `git init`. These are redundant — the `git()` helper already injects `GIT_AUTHOR_*` / `GIT_COMMITTER_*` env vars for every invocation, which is how `init_repo()` handles identity. Dropping these calls has no effect on test behaviour.

**Workflow divergence (benign):** The old helper used a hand-rolled 5-state workflow (new, specd, ready, in_progress, closed). After migration the repo carries the full production workflow (~12 states). All 6 tests exercise only `apm new`, `apm next`, `apm list`, `apm close`, `apm spec`, and `apm set` — none of which are sensitive to extra workflow states. No test changes are expected.

**Verification:** Run `cargo test --test integration -- aggressive` to confirm all 6 tests pass:
- `aggressive_no_remote_does_not_abort_next`
- `aggressive_no_remote_does_not_abort_list`
- `aggressive_no_remote_does_not_abort_close`
- `no_aggressive_flag_suppresses_fetch_on_next`
- `no_aggressive_flag_suppresses_fetch_on_spec`
- `no_aggressive_flag_suppresses_fetch_on_set`

**Dependency note:** This ticket depends on 795dce11 (`init_repo()` helper). Implement only after 795dce11 is closed and `init_repo()` is available in `integration.rs`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:39Z | groomed | in_design | philippepascal |
| 2026-05-02T03:44Z | in_design | specd | claude-0502-0339-1d40 |
| 2026-05-03T20:16Z | specd | ready | philippepascal |
