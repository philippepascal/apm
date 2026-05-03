+++
id = "a0171e83"
title = "Migrate epic-related test setups to init_repo() + real apm epic"
state = "ready"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a0171e83-migrate-epic-related-test-setups-to-init"
created_at = "2026-05-01T20:27:07.814641Z"
updated_at = "2026-05-03T20:17:06.424857Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Four setup helpers in `apm/tests/integration.rs` build their fixture repos without calling `apm init`, so changes to the production init template — default workflow states, config file layout, `.gitignore` entries — are invisible to the 9 tests that depend on them.

- **`setup_with_epic()` (line 2535)** delegates to `setup()`, which hand-writes `apm.toml` at the repo root. The epic branch is created with raw `git checkout -b epic/<id>-…` calls.
- **`setup_with_epic_for_owner_tests()` (line 5460)** is a thin wrapper around `setup_with_epic()` that adds `.apm/local.toml`; it inherits the same problem.
- **`setup_epic_list()` (line 4311)** and **`setup_epic_show()` (line 4431)** each build a fresh tempdir, write a hard-coded 3-state `apm.toml` (ready / implemented / closed), and commit it — never calling `apm init`.

In all four cases the fixture diverges from what real users get: the config lives at the legacy `apm.toml` path instead of `.apm/config.toml`, the workflow is a frozen subset of the production default, and any addition of a required field or state to the init template will not surface in these tests.

The desired state is that all four helpers use `init_repo()` for their repo scaffolding. Because `apm epic new` requires a remote origin (it runs `git fetch` and pushes the new branch), direct epic branch creation via git must remain — but must be marked `// BYPASS:` per the epic's bypass policy, making the workaround explicit and searchable.

### Acceptance criteria

- [ ] `setup_with_epic()` calls `init_repo()` instead of `setup()`
- [ ] `setup_epic_list()` body is replaced entirely by `init_repo()`
- [ ] `setup_epic_show()` body is replaced entirely by `init_repo()`
- [ ] `setup_with_epic_for_owner_tests()` requires no structural change: it inherits `init_repo()` via its call to `setup_with_epic()`
- [ ] The inline epic branch creation inside `setup_with_epic()` carries a `// BYPASS: apm epic new requires a remote origin` comment
- [ ] `create_epic_branch()` carries a `// BYPASS: apm epic new requires a remote origin` comment at its first git line
- [ ] No `apm.toml` hand-write or `git init` call remains in any of the four helpers
- [ ] All 9 tests that call these helpers pass: `new_epic_sets_frontmatter_fields`, `new_epic_branch_created_from_epic_tip`, `epic_list_no_epics_exits_zero_no_output`, `epic_list_shows_epics_with_derived_state_and_counts`, `epic_show_displays_header_and_ticket_table`, `epic_show_prefix_resolves_correctly`, `epic_bulk_owner_change_succeeds`, `epic_bulk_owner_change_skips_closed`, `epic_bulk_owner_change_blocked_non_owner`

### Out of scope

- Replacing `write_ticket_in_epic()` or `commit_ticket_to_branch()` direct TOML writes with `apm new` — covered by ticket 059e2e74
- Migrating any other helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in the epic
- Making `apm epic new` work without a remote origin — that is a product-feature decision
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Adding a `// BYPASS:` annotation anywhere outside the four helpers and `create_epic_branch()` named in this ticket

### Approach

All changes are in `apm/tests/integration.rs`. The dependency ticket 795dce11 must be merged first so `init_repo()` exists in the file.

**State compatibility check (done at spec time)**

The production `workflow.toml` (`.apm-core/src/default/workflow.toml`) includes `ready`, `implemented` (with `satisfies_deps = true`), and `closed` (with `terminal = true`) — exactly the states the epic list and epic show tests depend on for derived-state assertions (`in_progress`, `implemented`, `empty`, `done`). No state aliases or config overrides are needed.

---

**1. `setup_with_epic()` (line 2535) — replace `setup()` with `init_repo()`**

Replace the function body:

```rust
fn setup_with_epic() -> (tempfile::TempDir, String) {
    let dir = init_repo();
    let p = dir.path();
    let epic_id = "ab12cd34";
    let epic_branch = format!("epic/{epic_id}-my-epic");
    // BYPASS: apm epic new requires a remote origin; create epic branch directly via git
    git(p, &["checkout", "-b", &epic_branch]);
    std::fs::write(p.join("EPIC.md"), "# my-epic\n").unwrap();
    git(p, &["add", "EPIC.md"]);
    git(p, &["commit", "-m", &format!("epic({epic_id}): create my-epic")]);
    git(p, &["checkout", "main"]);
    (dir, epic_id.to_string())
}
```

Drop the old `git(p, &["config", "user.email", …])` and `git(p, &["config", "user.name", …])` lines — `init_repo()` handles those via the `git()` helper's injected env vars. Drop the `-c commit.gpgsign=false` flags for the same reason (the `git()` helper already sets `GIT_AUTHOR_*` / `GIT_COMMITTER_*`).

**2. `setup_with_epic_for_owner_tests()` (line 5460) — no structural change**

This function calls `setup_with_epic()` and writes `.apm/local.toml`. After step 1, `.apm/` already exists (created by `apm init`), so `create_dir_all` is harmless. No edits required.

**3. `setup_epic_list()` (line 4311) — replace entire body**

```rust
fn setup_epic_list() -> TempDir {
    init_repo()
}
```

Remove the hand-written `apm.toml`, the `git init`, `git config`, and `create_dir_all("tickets")` calls. `init_repo()` handles all of them.

**4. `setup_epic_show()` (line 4431) — replace entire body**

```rust
fn setup_epic_show() -> tempfile::TempDir {
    init_repo()
}
```

Same rationale as step 3.

**5. `create_epic_branch()` (line 4355) — add BYPASS comment**

Add the comment as the first line inside the function body:

```rust
fn create_epic_branch(dir: &std::path::Path, branch: &str) {
    // BYPASS: apm epic new requires a remote origin; create epic branch directly via git
    git(dir, &["checkout", "-b", branch]);
    …
}
```

No other lines in `create_epic_branch` change.

---

**Verification**

Run `cargo test --test integration` and confirm all 9 affected tests pass. No other tests should regress because the four helpers are only called by those 9 tests.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:56Z | groomed | in_design | philippepascal |
| 2026-05-02T04:02Z | in_design | specd | claude-0502-0356-0e78 |
| 2026-05-03T20:17Z | specd | ready | philippepascal |
