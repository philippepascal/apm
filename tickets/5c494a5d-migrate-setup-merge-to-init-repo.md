+++
id = "5c494a5d"
title = "Migrate setup_merge() to init_repo()"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5c494a5d-migrate-setup-merge-to-init-repo"
created_at = "2026-05-01T20:26:46.198163Z"
updated_at = "2026-05-02T03:28:06.479201Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_merge()` at `apm/tests/integration.rs:134` hand-writes an `apm.toml` at the repo root containing a 7-state workflow and `completion = "merge"` on the `in_progress → implemented` transition. It never calls `apm init`, so the fixture diverges from the production repo shape: the file is written to the legacy `apm.toml` location (not `.apm/workflow.toml`), the state list is smaller than the production default, and changes to the init template — new states, field renames, config file layout — are invisible to the 6 tests that depend on this helper.

The `merge` completion strategy is intentional and must be preserved. The 6 `depends_on` tests create tickets with no epic, relying on identical `target_branch` values (both defaulting to `main`) to satisfy validation. The production default `completion = "pr_or_epic_merge"` would reject those tickets because they lack an epic. There is no `apm` command to change a workflow completion strategy post-init, so a `// BYPASS:` filesystem edit is required after `init_repo()` runs.

### Acceptance criteria

- [ ] `setup_merge()` calls `init_repo()` and no longer contains any hand-written TOML string or `std::fs::write` for `apm.toml` / `config.toml` / `workflow.toml`
- [ ] The rewritten helper includes a `// BYPASS:` comment explaining that no `apm` command can set a completion strategy post-init
- [ ] `.apm/workflow.toml` in the returned repo has `completion = "merge"` on the `in_progress → implemented` transition
- [ ] The patched `.apm/workflow.toml` is committed before `setup_merge()` returns (working tree is clean)
- [ ] `set_depends_on_single_id` passes
- [ ] `set_depends_on_comma_separated` passes
- [ ] `set_depends_on_clear` passes
- [ ] `set_depends_on_trims_whitespace` passes
- [ ] `new_depends_on_sets_frontmatter` passes
- [ ] `new_depends_on_comma_separated` passes
- [ ] No other test in `integration.rs` regresses

### Out of scope

- Migrating any other helper (`setup()`, `setup_with_close_workflow()`, `setup_aggressive()`, etc.) — each has its own sibling ticket in this epic
- Changing any test function that calls `setup_merge()` — only the helper body is in scope
- Adding an `apm` command to configure completion strategy post-init — that is a product feature decision
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Migrating `setup_merge_strategy_remote()` (line 5301) — that is a distinct helper unrelated to `setup_merge()`

### Approach

Replace the body of `setup_merge()` at `apm/tests/integration.rs:134–236`. The function signature (`fn setup_merge() -> TempDir`) and all callers are unchanged.

**New body — ordered steps**

1. Call `init_repo()` to obtain a fully initialised repo with `.apm/config.toml`, `.apm/workflow.toml`, `tickets/`, `.gitignore`, and a committed HEAD.

2. Read `.apm/workflow.toml`, apply the BYPASS, and write it back:
   ```rust
   // BYPASS: no apm command can change a workflow completion strategy post-init.
   // Change pr_or_epic_merge → merge so that depends_on is allowed for tickets
   // that share the same target_branch without belonging to an epic.
   let wf_path = dir.path().join(".apm/workflow.toml");
   let wf = std::fs::read_to_string(&wf_path).unwrap();
   assert!(
       wf.contains("completion = \"pr_or_epic_merge\""),
       "expected pr_or_epic_merge in workflow.toml — default template may have changed"
   );
   let patched = wf.replace(
       "completion = \"pr_or_epic_merge\"",
       "completion = \"merge\"",
   );
   std::fs::write(&wf_path, patched).unwrap();
   ```
   The `assert!` guards against silent drift if the default template is ever renamed; a clear panic message is better than a subtly wrong test fixture.

3. Stage and commit the patched file so HEAD is clean when the function returns:
   ```rust
   git(dir.path(), &["add", ".apm/workflow.toml"]);
   git(dir.path(), &["commit", "-m", "set merge completion"]);
   ```

4. Return `dir`.

**Why `pr_or_epic_merge` → `merge` is the only change needed**

The 6 callers create tickets with `apm new` (no epic, no explicit `target_branch`), then call `apm set depends_on` or pass `depends_on` to `apm new`. The validation in `apm-core/src/validate.rs` (`check_depends_on_rules`) only inspects the completion field:
- `merge` → allows `depends_on` when all deps share the same `target_branch` (both default to `main` ✓)
- `pr_or_epic_merge` → requires all deps to belong to the same epic (none do ✗)

The extra states in the production workflow (`groomed`, `in_design`, `ready`, `blocked`, `merge_failed`, etc.) are irrelevant: none of the 6 tests advance tickets past `new`, and `depends_on` validation does not gate on current state.

**File changed**

`apm/tests/integration.rs` — lines 134–236 replaced in full (the `setup_merge` function body only).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:22Z | groomed | in_design | philippepascal |