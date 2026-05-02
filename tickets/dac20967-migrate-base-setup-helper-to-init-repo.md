+++
id = "dac20967"
title = "Migrate base setup() helper to init_repo()"
state = "specd"
priority = 0
effort = 6
risk = 4
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/dac20967-migrate-base-setup-helper-to-init-repo"
created_at = "2026-05-01T20:26:43.905437Z"
updated_at = "2026-05-02T03:22:25.576970Z"
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

- Migrating any other setup helper (`setup_merge()`, `setup_with_close_workflow()`, `setup_aggressive()`, etc.) — each is covered by a dedicated sibling ticket in this epic
- Changing the behaviour of `apm init` itself
- The `init_repo()` implementation (covered by upstream ticket 795dce11)
- Adding new tests for the production-only states (`groomed`, `in_design`, `blocked`, `implemented`, `merge_failed`) — those belong to feature tickets
- Removing the `apm.toml` legacy fallback from `Config::load` (covered by ticket 40fdde3b, intentionally last in the epic)
- CI enforcement or linting of the bypass policy (covered by ticket 8217e5f5)

### Approach

**File:** `apm/tests/integration.rs`

**Step 1 — Replace `setup()` body**

Remove the entire body of `setup()` (lines 34–129: git init, git config calls, `std::fs::write` for `apm.toml`, git add/commit, `create_dir_all` for `tickets/`) and replace it with a single delegation to `init_repo()`:

```rust
fn setup() -> TempDir {
    init_repo()
}
```

`init_repo()` (from ticket 795dce11) already handles: tempdir creation, `git init -q -b main`, `apm init --no-claude --quiet`, initial commit. `tickets/` is created by `apm init` itself.

**Step 2 — Run the test suite and triage failures**

Run `cargo test --test integration 2>&1 | grep -E "^(test |FAILED|error)"` to collect all failing tests.

Failures will fall into three categories:

**Category A — Invalid transition (e.g. `new → specd`, `new → ready`, `new → in_progress`)**

These tests call `apm::cmd::state::run(p, &id, "specd".into(), false, false)` (or similar) directly from `new`. Production workflow does not allow these direct hops. Fix: pass `force: true` as the last argument. `--force` still validates that the target state exists in the config and (for `specd`) that a valid spec is present, so the test remains meaningful.

```rust
// Before
apm::cmd::state::run(p, &id, "specd".into(), false, false).unwrap();
// After
apm::cmd::state::run(p, &id, "specd".into(), false, true).unwrap();
```

**Category B — Test checks workflow structure (state list, transition list, or state count)**

Tests that assert things like "workflow has 6 states" or "valid transitions from `new` are `[specd, ready, in_progress, closed]`" must be updated to reflect the production 12-state workflow. Update the expected values; do not delete these tests — they are now exercising the real workflow shape.

**Category C — Test scenario no longer makes sense**

If a test was written specifically to exercise a workflow edge that only existed in the 6-state fixture (e.g., a test that verified a state name unique to the old fixture), delete the test and add a `// DELETED: <reason>` comment at the callsite. Examples of likely deletions: tests checking that exactly 6 states exist, or tests asserting specific transition behaviour that was an artefact of the simplified fixture.

**Step 3 — Verify no `apm.toml` at root**

After migration, the config lives at `.apm/config.toml` (produced by `apm init`). Any test that opens `p.join("apm.toml")` directly must be updated to `p.join(".apm/config.toml")`. If such a direct open is legitimately testing legacy-fallback behaviour, mark it `// BYPASS: testing legacy apm.toml fallback` and note it will be removed with ticket 40fdde3b.

**Step 4 — Final check**

`cargo test --test integration` must exit 0. Confirm `setup()` contains no `std::fs::write` and no git config calls.

**Known transition delta (production workflow)**

Transitions valid in the old 6-state fixture but NOT in production (require `force: true` or path update):
- `new → specd`
- `new → ready`
- `new → in_progress`
- `new → ammend`

Transitions valid in both (no change needed for these):
- `new → closed`
- `specd → ready`
- `specd → ammend`
- `specd → closed`
- `ready → in_progress` (production uses `command:start` trigger, but `force: false` still works for manual calls in tests)
- `in_progress → closed`
- `ammend → specd`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:17Z | groomed | in_design | philippepascal |
| 2026-05-02T03:22Z | in_design | specd | claude-0502-0317-5c38 |
