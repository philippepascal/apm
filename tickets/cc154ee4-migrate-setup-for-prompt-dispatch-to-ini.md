+++
id = "cc154ee4"
title = "Migrate setup_for_prompt_dispatch() to init_repo()"
state = "ready"
priority = 0
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/cc154ee4-migrate-setup-for-prompt-dispatch-to-ini"
created_at = "2026-05-01T20:27:03.975333Z"
updated_at = "2026-05-03T20:17:03.409749Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_for_prompt_dispatch()` at `apm/tests/integration.rs:2099` hand-rolls a 6-state workflow (`new`, `in_design`, `ammend`, `ready`, `in_progress`, `closed`) with `trigger = "command:start"` transitions on `new`, `ammend`, and `ready`. It writes the config to the legacy `apm.toml` root location, never calls `apm init`, and creates the `.apm/` directory manually.

This diverges from the production repo shape in two ways. First, the config file is at the wrong location (`apm.toml` instead of `.apm/config.toml`). Second, the `new` state having `command:start → in_design` is a custom invention — in production, the dispatch path for spec-writing is `groomed → in_design`. Tests `spawn_new_ticket_transitions_to_in_design` and `start_next_spawn_new_ticket_transitions_correctly` therefore exercise dispatch against a non-production state, masking any breakage in the real `groomed` dispatch path.

There are 7 tests that depend on this helper. They cover owner-preservation semantics on `in_design` transitions, and the prompt-dispatch mechanism for `ammend → in_design`, `ready → in_progress`, and the `groomed → in_design` path (currently exercised via the ersatz `new` state).

### Acceptance criteria

- [ ] `setup_for_prompt_dispatch()` no longer writes `apm.toml`; it calls `init_repo()` as its first step
- [ ] The test repo produced by `setup_for_prompt_dispatch()` has `.apm/config.toml` (not `apm.toml`) as its config file
- [ ] The mock worker path is injected into `.apm/config.toml` so that `apm start --spawn` can invoke it
- [ ] The injection is marked with a `// BYPASS:` comment explaining why direct file editing is used
- [ ] `spawn_new_ticket_transitions_to_in_design` passes using a `groomed`-state ticket (matching the production dispatch path)
- [ ] `start_next_spawn_new_ticket_transitions_correctly` passes using a `groomed`-state ticket
- [ ] `spawn_ammend_ticket_transitions_to_in_design` passes unchanged (production workflow already has `ammend → in_design` via `command:start`)
- [ ] `spawn_ready_ticket_transitions_to_in_progress` passes unchanged (production workflow already has `ready → in_progress` via `command:start`)
- [ ] `start_next_spawn_ready_ticket_transitions_correctly` passes unchanged
- [ ] `in_design_does_not_set_owner_when_unowned` passes unchanged
- [ ] `in_design_does_not_overwrite_different_owner` passes unchanged
- [ ] All 7 tests pass under `cargo test` with no modifications to test assertions

### Out of scope

- Migrating any other setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in this epic
- Replacing `write_ticket_to_branch()` / `write_ticket_with_owner()` direct file writes with real `apm new` + `apm state` calls — covered by sibling ticket 059e2e74
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Adding a CLI command to configure `workers.command` post-init — that is a product feature decision
- Changing any test assertion or the behavior being tested — only the fixture setup is in scope

### Approach

All changes are in `apm/tests/integration.rs`.

**1. Rewrite `setup_for_prompt_dispatch()` (line 2099)**

Replace the entire function body with:

```rust
fn setup_for_prompt_dispatch() -> TempDir {
    let dir = init_repo();
    let p = dir.path();
    let mock_worker = make_mock_worker(p);

    // BYPASS: no apm CLI command to set workers.command; inject directly into
    // .apm/config.toml which apm init has already created at this location.
    let config_path = p.join(".apm/config.toml");
    let cfg = std::fs::read_to_string(&config_path).unwrap();
    let patched = cfg.replace(
        "[workers]\n",
        &format!("[workers]\ncommand = \"{}\"\n", mock_worker.display()),
    );
    std::fs::write(&config_path, patched).unwrap();

    dir
}
```

`init_repo()` (from ticket 795dce11) already:
- creates the tempdir, runs `git init`, invokes `apm init --no-claude --quiet`
- commits all generated files so HEAD resolves
- produces `.apm/config.toml`, `.apm/workflow.toml`, `tickets/`, and `.gitignore`

The TOML patch inserts `command = "..."` immediately after the `[workers]` section header that `apm init` writes (`[workers]\nagent = "claude"`). Both fields coexist in `WorkersConfig`; `command` takes dispatch priority over `agent`.

Do **not** re-add `create_dir_all` calls for `tickets/` or `.apm/` — `init_repo()` provides both.

**2. Update two test call sites that create `"new"`-state tickets for dispatch**

The production workflow dispatches spec-writing tickets from the `groomed` state, not `new`. Change the state argument from `"new"` to `"groomed"` at:

- Line 2188 (`spawn_new_ticket_transitions_to_in_design`):
  ```
  // before
  write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me");
  // after
  write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "groomed", 1, "spec me");
  ```

- Line 2231 (`start_next_spawn_new_ticket_transitions_correctly`):
  ```
  // before
  write_ticket_with_owner(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me", "test-agent");
  // after
  write_ticket_with_owner(p, "ticket/0001-spec-me", "0001-spec-me.md", "groomed", 1, "spec me", "test-agent");
  ```

The production `workflow.toml` generated by `apm init` has `groomed → in_design` via `trigger = "command:start"` (with profile `spec_agent`). Both tests assert `state = "in_design"` after `apm start --spawn`, which remains correct.

**3. No changes needed to the other 5 tests**

- Tests using `"ammend"` and `"ready"` states: production workflow already has `command:start` on both. No change.
- Tests 1 and 2 (`in_design_does_not_set_owner_*`): they call `state::run` with `force = true`, which bypasses the workflow DAG. The `new` and `in_design` states both exist in the production workflow, so the forced transition continues to work.
- Tests 6 and 7 write `username = "test-agent"` to `.apm/local.toml` after setup. Since `workers.command` is now in `config.toml` (not `local.toml`), these writes no longer risk clobbering the workers config.

**4. Commit**

Single commit: `ticket(cc154ee4): migrate setup_for_prompt_dispatch to init_repo()`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:49Z | groomed | in_design | philippepascal |
| 2026-05-02T03:56Z | in_design | specd | claude-0502-0349-0838 |
| 2026-05-03T20:17Z | specd | ready | philippepascal |
