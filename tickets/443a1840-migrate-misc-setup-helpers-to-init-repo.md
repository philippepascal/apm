+++
id = "443a1840"
title = "Migrate misc setup helpers to init_repo()"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/443a1840-migrate-misc-setup-helpers-to-init-repo"
created_at = "2026-05-01T20:27:23.868607Z"
updated_at = "2026-05-02T04:24:51.214180Z"
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

- [ ] `setup_with_satisfies_deps()` body is replaced with `init_repo()` only — no `git init`, no `std::fs::write`, no `apm.toml` reference remains
- [ ] `next_skips_dep_blocked_returns_unblocked` passes
- [ ] `next_returns_dep_blocked_after_dep_satisfies` passes
- [ ] `next_picks_low_priority_blocker_before_higher_raw_independent` passes
- [ ] `setup_with_archive_dir()` body is replaced with `init_repo()` only — no `apm.toml` read/write remains
- [ ] All 6 tests calling `setup_with_archive_dir` pass
- [ ] `setup_with_server_url(url)` calls `init_repo()` and appends the `[server]` block to `.apm/config.toml` (not `apm.toml`)
- [ ] The `[server]` block injection in `setup_with_server_url` carries a `// BYPASS: no apm command configures server.url` comment
- [ ] All 7 tests calling `setup_with_server_url` pass
- [ ] `setup_on_failure_fix_project` calls `init_repo()` instead of hand-writing git init, config.toml, and workflow.toml
- [ ] Each filesystem mutation in `setup_on_failure_fix_project` (stripping `on_failure` line, removing `merge_failed` block) carries a `// BYPASS:` comment explaining why no apm command can do it
- [ ] `test_fix_adds_field_only` passes
- [ ] `test_fix_adds_state_only` passes
- [ ] `test_fix_adds_both_atomically` passes
- [ ] `test_fix_is_idempotent` passes
- [ ] No test function body is changed — only the four helper bodies are modified

### Out of scope

- Migrating `commit_ticket_to_branch()` — utility function, not a config-carrying setup helper
- Migrating `setup_epic_list()` (line 4311) or any other helper not listed in the Problem section
- Migrating `setup()`, `setup_merge()`, `setup_with_close_workflow()`, `setup_with_local_worktrees()`, `setup_with_worktrees()` — each has a dedicated sibling ticket in the epic
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Adding `apm` commands to set `server.url`, remove workflow states, or toggle `archive_dir`
- Changing any test function body (only the four helper bodies are in scope)

### Approach

All four helpers live in `apm/tests/integration.rs`. The dependency ticket 795dce11 adds `init_repo()` near the top of that file (after `git()`, before `setup()`); assume it is present.

---

### `setup_with_satisfies_deps` (line 4156)

Replace the entire body with `init_repo()`. No BYPASS required.

**Why it works:** The production default workflow (`apm-core/src/default/workflow.toml`) already has `satisfies_deps = true` on `implemented` and `actionable = ["agent"]` on `ready` — the two properties the three dependent tests rely on. Prioritization weights (`priority_weight = 10.0`, `effort_weight = -2.0`, `risk_weight = -1.0`) match the Rust-side defaults, so the score ordering in `next_picks_low_priority_blocker_before_higher_raw_independent` is unaffected. The `tickets/` directory is created by `apm init`; `commit_ticket_to_branch()` also calls `create_dir_all("tickets")`, so nothing breaks.

Result:
```rust
fn setup_with_satisfies_deps() -> TempDir {
    init_repo()
}
```

---

### `setup_with_archive_dir` (line 5101)

Replace the entire body with `init_repo()`. No BYPASS required.

**Why it works:** The default config template (`apm-core/src/init.rs`, `default_config()`) already emits `archive_dir = "archive/tickets"` under `[tickets]`. So `Config::load()` on an `init_repo()` repo already returns a config with `archive_dir` set. All 6 dependent tests use `apm::cmd::new::run`, `apm::cmd::close::run`, and `apm_core::archive::archive`, which all work correctly with the production 12-state workflow (the `closed` terminal state is present).

Result:
```rust
fn setup_with_archive_dir() -> TempDir {
    init_repo()
}
```

---

### `setup_with_server_url(url: &str)` (line 4854)

1. Replace `let dir = setup()` with `let dir = init_repo()`.
2. Change the config file path from `p.join("apm.toml")` to `p.join(".apm/config.toml")`.
3. Keep the append of `\n[server]\nurl = "..."` but mark it BYPASS — there is no `apm` command that sets `server.url` in the config file.

Result:
```rust
fn setup_with_server_url(url: &str) -> TempDir {
    let dir = init_repo();
    let p = dir.path();
    // BYPASS: no apm command configures server.url — append directly to config
    let config_path = p.join(".apm/config.toml");
    let existing = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(&config_path, format!("{existing}\n[server]\nurl = \"{url}\"\n")).unwrap();
    dir
}
```

---

### `setup_on_failure_fix_project(on_failure, declare_merge_failed)` (line 2852)

The helper intentionally creates a *partially broken* workflow so the `validate --fix` tests can verify repair behaviour. `init_repo()` produces a fully correct workflow; the BYPASS mutations selectively remove the pieces that should be absent.

**Production workflow facts** (from `apm-core/src/default/workflow.toml`):
- The `in_progress -> implemented` transition carries `completion = "pr_or_epic_merge"` and `on_failure = "merge_failed"`.
- The `merge_failed` state block exists and is the penultimate `[[workflow.states]]` block (position 11 of 12, before `closed`).

Steps:

1. Call `init_repo()`, bind to `dir`; get `p = dir.path()`.
2. Read `.apm/workflow.toml` into `content: String`.
3. **If `on_failure.is_none()`:**
   - BYPASS: no `apm` command removes `on_failure` from a workflow transition.
   - Strip every line whose trimmed form starts with `on_failure` (using `.lines().filter(...).collect::<Vec<_>>().join("\n")`).
4. **If `!declare_merge_failed`:**
   - BYPASS: no `apm` command removes a workflow state.
   - Remove the entire `merge_failed` state block. Strategy: split `content` on `"[[workflow.states]]"` boundaries, filter out the segment whose `id` line contains `"merge_failed"`, then rejoin. Alternatively, scan lines and set a `skip` flag from the `[[workflow.states]]` header where the following `id` line contains `"merge_failed"` until the next `[[workflow.states]]` header.
5. Write the (possibly modified) content back to `.apm/workflow.toml`.
6. Return `dir`. No additional git commit is needed — `validate::run()` reads from the filesystem, not from git history for workflow config.

**Caller map:**

| Test | on_failure | declare_merge_failed | Mutations |
|------|-----------|---------------------|-----------|
| `test_fix_adds_field_only` | `None` | `true` | strip `on_failure` lines only |
| `test_fix_adds_state_only` | `Some("merge_failed")` | `false` | strip `merge_failed` block only |
| `test_fix_adds_both_atomically` | `None` | `false` | both mutations |
| `test_fix_is_idempotent` | `None` | `false` | both mutations |

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:17Z | groomed | in_design | philippepascal |