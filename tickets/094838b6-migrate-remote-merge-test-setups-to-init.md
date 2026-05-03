+++
id = "094838b6"
title = "Migrate remote-merge test setups to init_repo()"
state = "ready"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/094838b6-migrate-remote-merge-test-setups-to-init"
created_at = "2026-05-01T20:27:20.736073Z"
updated_at = "2026-05-03T20:17:12.511488Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Three setup helpers in `apm/tests/integration.rs` back the merge-strategy integration tests:
`setup_squash_remote` (line 3914), `setup_pr_or_epic_merge_remote` (line 4710), and
`setup_merge_strategy_remote` (line 5301). All three follow the same bare-remote + local-clone
pattern, but each hand-writes a minimal `apm.toml` at repo root and never calls `apm init`.

As a result each fixture diverges from a real user repo in two ways:

1. Config is written to the legacy `apm.toml` location instead of `.apm/config.toml` /
   `.apm/workflow.toml`.
2. The hand-crafted workflow states are a small, frozen subset of the production default
   (3–4 states vs. the 12-state production default).

Any change to the init template — new required states, renamed fields, config file layout —
is invisible to the 4 tests backed by these helpers. The helpers should instead call the real
`apm init` binary (same pattern established by dependency ticket 795dce11's `init_repo()`
helper) and override only what the test needs via real commands or marked bypass.

### Acceptance criteria

- [ ] `setup_squash_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] `setup_pr_or_epic_merge_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] `setup_merge_strategy_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] A new private helper `init_remote_repo() -> (TempDir, TempDir)` exists that creates a bare remote + local clone via `apm init --no-claude --quiet`
- [ ] `squash_merge_config()`, `pr_or_epic_merge_config_toml()`, and `merge_strategy_config_toml()` are removed (they become dead code after migration)
- [ ] The squash-merge detection tests `sync_detect_squash_merged_branch_remote_ref_present` and `sync_detect_squash_merged_branch_remote_ref_deleted` pass
- [ ] The pr_or_epic_merge tests `pr_or_epic_merge_with_target_branch_merges_into_target`, `pr_or_epic_merge_without_target_branch_attempts_pr`, and `pr_or_epic_merge_with_target_branch_pushes_target_to_origin` pass
- [ ] The merge-strategy test `merge_strategy_merges_locally_without_push` passes
- [ ] `setup_merge_strategy_remote()` includes a `// BYPASS:` comment explaining why `.apm/workflow.toml` is edited directly
- [ ] The bypass in `setup_merge_strategy_remote()` edits `.apm/workflow.toml` (not `apm.toml`)

### Out of scope

- Migrating `write_implemented_ticket` or `write_in_progress_ticket` to use `apm new` / `apm state` — those are ticket-content helpers covered by sibling ticket 059e2e74
- Adding an `apm` CLI command to configure completion strategy on a specific transition — product feature decision, not in scope here
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Migrating any other setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in this epic
- Changing any test function body (only the three setup helper bodies are in scope)
- Migrating `push_to_origin()`, `remote_ref_sha()`, or `local_ref_sha()` — those are support utilities, not config-carrying setup helpers
- Adding the `init_repo()` helper (covered by dependency ticket 795dce11); `init_remote_repo()` follows the same pattern but is a separate bare+clone variant

### Approach

**Key insight from reading the code:** The production default `workflow.toml` (written by
`apm init`) already has `completion = "pr_or_epic_merge"` on the `in_progress → implemented`
transition. Only `setup_merge_strategy_remote` needs a bypass (to change it to
`completion = "merge"`). The other two helpers need no completion strategy override at all —
the production default is sufficient.

---

### Step 1 — Add `fn init_remote_repo() -> (TempDir, TempDir)`

Place it just before `setup_squash_remote` (near the `// --- squash-merge detection ---`
comment, around line 3912). This is the shared core for all three migrated helpers.

```rust
fn init_remote_repo() -> (TempDir, TempDir) {
    let bare = tempfile::tempdir().unwrap();
    let bp = bare.path();
    git(bp, &["init", "--bare", "-q"]);

    let local = tempfile::tempdir().unwrap();
    let p = local.path();
    git(p, &["clone", "-q", &bp.to_string_lossy(), "."]);

    let bin = env!("CARGO_BIN_EXE_apm");
    let out = std::process::Command::new(bin)
        .args(["init", "--no-claude", "--quiet"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "apm init failed: {}", String::from_utf8_lossy(&out.stderr));

    git(p, &["add", "."]);
    git(p, &["commit", "-m", "init"]);
    git(p, &["push", "origin", "main"]);

    (bare, local)
}
```

Notes:
- `git clone -q` suppresses the "empty repository" warning.
- `apm init` runs `maybe_initial_commit` internally (commits `.apm/config.toml`,
  `.apm/workflow.toml`, `.apm/ticket.toml`, `.gitignore`). The subsequent
  `git add . && git commit` captures any remaining files (`CLAUDE.md`,
  `.apm/agents.md`, `.apm/apm.spec-writer.md`, `.apm/apm.worker.md`). If
  `maybe_initial_commit` staged everything first, `git commit` exits non-zero
  and `git()` silently ignores it (`.status().unwrap()` only panics on I/O error,
  not on non-zero exit code).
- `git push origin main` uses the explicit remote+branch form, so no upstream
  tracking configuration is required.
- No manual `git config user.email/name` needed — the `git()` helper already
  injects `GIT_AUTHOR_*` / `GIT_COMMITTER_*`.
- No `tickets/` mkdir needed — `apm init` creates `tickets/` automatically.

---

### Step 2 — Rewrite `setup_squash_remote()`

The squash-detection tests only need `Config::load` to succeed and find
`default_branch = "main"`. The production default workflow already has `new`,
`implemented`, and `closed` states, which is all `sync::detect` needs (it checks
`terminal_state_ids()` and `state == "implemented"` by hard-coded string). No bypass
required.

Replace the entire body:

```rust
fn setup_squash_remote() -> (TempDir, TempDir) {
    init_remote_repo()
}
```

Remove the now-dead `squash_merge_config()` function.

---

### Step 3 — Rewrite `setup_pr_or_epic_merge_remote()`

The production default `workflow.toml` already declares the `in_progress → implemented`
transition with `completion = "pr_or_epic_merge"` and `on_failure = "merge_failed"`.
The `on_failure` field is present in the production default but was absent in the
old hand-written config; this does not affect the existing tests:

- `pr_or_epic_merge_with_target_branch_merges_into_target`: merge succeeds → `on_failure`
  is never consulted.
- `pr_or_epic_merge_without_target_branch_attempts_pr`: the no-target-branch code path
  calls `gh_pr_create_or_update(…)?` and propagates errors directly with `?`, bypassing
  `on_failure` entirely — the test still gets `Err`.

Replace the entire body:

```rust
fn setup_pr_or_epic_merge_remote() -> (TempDir, TempDir) {
    init_remote_repo()
}
```

Remove the now-dead `pr_or_epic_merge_config_toml()` function.

---

### Step 4 — Rewrite `setup_merge_strategy_remote()`

The production default has `completion = "pr_or_epic_merge"` but this helper needs
`completion = "merge"`. No `apm` CLI command exists to override a transition's
completion strategy, so a bypass is required.

The bypass reads `.apm/workflow.toml`, replaces the single occurrence of
`completion = "pr_or_epic_merge"` with `completion = "merge"`, and writes it back.
The `on_failure = "merge_failed"` line is kept because `completion = "merge"` also
requires `on_failure` (enforced by `apm_core::validate`). The `merge_failed` state
is declared in the production default workflow, so config validation passes.

```rust
fn setup_merge_strategy_remote() -> (TempDir, TempDir) {
    let (bare, local) = init_remote_repo();
    let p = local.path();

    // BYPASS: no apm CLI command exists to override completion strategy on a
    // specific transition; edit .apm/workflow.toml directly to change
    // pr_or_epic_merge → merge on in_progress → implemented.
    // on_failure = "merge_failed" is kept — completion = "merge" also requires it.
    let wf_path = p.join(".apm/workflow.toml");
    let wf = std::fs::read_to_string(&wf_path).unwrap();
    let patched = wf.replace(r#"completion = "pr_or_epic_merge""#, r#"completion = "merge""#);
    std::fs::write(&wf_path, &patched).unwrap();
    git(p, &["add", ".apm/workflow.toml"]);
    git(p, &["commit", "-m", "override: use merge completion strategy"]);
    git(p, &["push", "origin", "main"]);

    (bare, local)
}
```

The replacement is unambiguous: `completion = "pr_or_epic_merge"` appears exactly once
in the default `workflow.toml`.

Remove the now-dead `merge_strategy_config_toml()` function.

---

### File changes summary

- `apm/tests/integration.rs`:
  - **Add** `fn init_remote_repo()` (just before `setup_squash_remote`, ~line 3912)
  - **Replace** `setup_squash_remote()` body with `init_remote_repo()` delegation
  - **Remove** `squash_merge_config()` static string function (~line 3882)
  - **Replace** `setup_pr_or_epic_merge_remote()` body with `init_remote_repo()` delegation
  - **Remove** `pr_or_epic_merge_config_toml()` static string function (~line 4687)
  - **Replace** `setup_merge_strategy_remote()` body with bypass-patched version
  - **Remove** `merge_strategy_config_toml()` static string function (~line 5278)

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:05Z | groomed | in_design | philippepascal |
| 2026-05-02T04:16Z | in_design | specd | claude-0502-0405-9e58 |
| 2026-05-03T20:17Z | specd | ready | philippepascal |
