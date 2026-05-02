+++
id = "c148f904"
title = "Migrate strict-transitions and merge-workflow setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c148f904-migrate-strict-transitions-and-merge-wor"
created_at = "2026-05-01T20:26:55.674729Z"
updated_at = "2026-05-02T03:35:03.756791Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_with_strict_transitions()` at `apm/tests/integration.rs:3747` and `setup_with_merge_workflow()` at `apm/tests/integration.rs:6845` each hand-write a full TOML config string to `apm.toml` at the repo root and never call `apm init`. The result is the same class of divergence as every other helper in this epic: the config lives at the legacy location, the repo has no committed HEAD before the helpers return, and changes to the production init template are invisible to the tests that depend on these fixtures.

Both helpers need custom workflow tables that differ structurally from the production default — not just a single field flip. `setup_with_strict_transitions()` needs `new → in_progress` as the only valid transition out of `new`, which the production workflow does not provide (`new` only goes to `groomed` or `closed`). `setup_with_merge_workflow()` needs a `new → implemented` transition carrying `completion = "merge"` and `on_failure = "merge_failed"`, which also does not exist in the production default (`in_progress → implemented` is the completion transition there). Neither delta can be achieved with a targeted `str::replace()` on the production workflow output; both require a full workflow overwrite via a marked `// BYPASS:` edit after `init_repo()` runs.

The two helpers serve 5 tests in total: `state_force_bypasses_transition_rules` and `state_force_implemented_from_in_progress` depend on the strict-transitions fixture; `merge_failure_transitions_ticket_to_merge_failed`, `merge_failed_to_implemented_does_not_trigger_another_merge`, and `merge_failed_to_in_progress_succeeds` depend on the merge-workflow fixture.

### Acceptance criteria

- [ ] **setup_with_strict_transitions()**

- [ ] `setup_with_strict_transitions()` calls `init_repo()` and no longer contains any hand-written TOML string or `std::fs::write` for `apm.toml` / `config.toml` / `workflow.toml`
- [ ] The rewritten helper includes a `// BYPASS:` comment explaining that the production workflow has no `new → in_progress` transition and a full replacement is required
- [ ] `.apm/workflow.toml` in the returned repo has `new → in_progress` as the only `[[workflow.states.transitions]]` block under the `new` state
- [ ] `.apm/workflow.toml` in the returned repo has no transition from `in_progress` to `new`
- [ ] The patched `.apm/workflow.toml` is committed before `setup_with_strict_transitions()` returns (working tree is clean)
- [ ] `state_force_bypasses_transition_rules` passes
- [ ] `state_force_implemented_from_in_progress` passes

- [ ] **setup_with_merge_workflow()**

- [ ] `setup_with_merge_workflow()` calls `init_repo()` and no longer contains any hand-written TOML string or `std::fs::write` for `apm.toml` / `config.toml` / `workflow.toml`
- [ ] The rewritten helper includes a `// BYPASS:` comment explaining that the production workflow has no `new → implemented` transition and a full replacement is required
- [ ] `.apm/workflow.toml` in the returned repo has a `new → implemented` transition with `completion = "merge"` and `on_failure = "merge_failed"`
- [ ] The patched `.apm/workflow.toml` is committed before `setup_with_merge_workflow()` returns (working tree is clean)
- [ ] `merge_failure_transitions_ticket_to_merge_failed` passes
- [ ] `merge_failed_to_implemented_does_not_trigger_another_merge` passes
- [ ] `merge_failed_to_in_progress_succeeds` passes
- [ ] No other test in `integration.rs` regresses

### Out of scope

- Migrating any other helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, `setup_aggressive()`, etc.) — each has its own sibling ticket in this epic
- Changing any test function body that calls `setup_with_strict_transitions()` or `setup_with_merge_workflow()` — only the helper bodies are in scope
- Adding an `apm` command to configure workflow transitions or completion strategies post-init — that is a product feature decision
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic

### Approach

Both helpers follow the same pattern: call `init_repo()`, overwrite `.apm/workflow.toml` with the custom table (BYPASS), commit the change, return `dir`. The function signatures are unchanged; all callers are unchanged.

**File changed:** `apm/tests/integration.rs` — two function bodies replaced (lines 3747–3804 and 6845–6909).

---

**setup_with_strict_transitions() — new body (lines 3747–3804)**

Replace the body of `setup_with_strict_transitions()`. The function signature is unchanged.

1. Call `init_repo()` to obtain a properly shaped repo (`.apm/` layout, `tickets/`, HEAD committed).

2. BYPASS — overwrite `.apm/workflow.toml` with the custom 5-state restricted-transition table:
   ```rust
   // BYPASS: no apm command can replace the workflow post-init.
   // The production default has no new → in_progress transition (new only goes to groomed
   // or closed); this 5-state restricted workflow is intentionally minimal to isolate
   // --force bypass behaviour without the full spec/review lifecycle.
   let wf = "[workflow]\n\n[[workflow.states]]\nid    = \"new\"\n...";
   std::fs::write(dir.path().join(".apm/workflow.toml"), wf).unwrap();
   ```
   The TOML content is the same 5-state table as the original fixture (new, specd, in_progress, implemented, closed) with exactly the same transitions. `specd` is kept to avoid state-membership panics if the parser validates state names on load.

3. Stage and commit the patched file:
   ```rust
   git(dir.path(), &["add", ".apm/workflow.toml"]);
   git(dir.path(), &["commit", "-m", "strict transitions workflow"]);
   ```

4. Return `dir`.

The key invariant preserved: `new → in_progress` is the only valid transition from `new`, and no `in_progress → new` transition exists. Both callers assert that `state::run` without `--force` rejects `in_progress → new` and that `--force` allows it.

---

**setup_with_merge_workflow() — new body (lines 6845–6909)**

Replace the body of `setup_with_merge_workflow()`. The function signature is unchanged.

1. Call `init_repo()` to obtain a properly shaped repo.

2. BYPASS — overwrite `.apm/workflow.toml` with the custom 5-state merge-workflow table:
   ```rust
   // BYPASS: no apm command can add a new → implemented transition with completion = "merge"
   // post-init. The production default has no such path; a direct new → implemented route
   // is required so tests can trigger merge-failure without the full spec/worktree lifecycle.
   std::fs::write(dir.path().join(".apm/workflow.toml"), wf).unwrap();
   ```
   The TOML content is the same table as the original fixture (states: new, implemented, merge_failed, in_progress, closed) with the `new → implemented` transition carrying `completion = "merge"` and `on_failure = "merge_failed"`.

3. Stage and commit:
   ```rust
   git(dir.path(), &["add", ".apm/workflow.toml"]);
   git(dir.path(), &["commit", "-m", "merge workflow"]);
   ```

4. Return `dir`.

---

**Dependency note**

Both helpers call `init_repo()`, which is defined by ticket 795dce11. That ticket is in `specd` state and must land on the epic branch first (or be implemented inline if the branches are merged). The `depends_on` field already captures this ordering.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:35Z | groomed | in_design | philippepascal |