+++
id = "c8bd9b55"
title = "Use mock-happy/mock-sad in integration tests instead of debug wrapper"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c8bd9b55-use-mock-happy-mock-sad-in-integration-t"
created_at = "2026-05-24T19:07:11.167447Z"
updated_at = "2026-05-24T20:34:14.954302Z"
+++

## Spec

### Problem

Integration tests that exercise worker spawning configure the `debug/worker` profile, which is a built-in no-op: it exits immediately without reading the ticket, writing spec sections, or calling `apm state`. Tests that use it only verify that `apm start --spawn` does not return an error and that the parent-side state transition (e.g. `groomed → in_design`, `ready → in_progress`) was written to the ticket branch. They say nothing about whether the agent loop itself — claim ticket, do work, call `apm state`, exit — functions end-to-end.

Two mock agents, `mock-happy` and `mock-sad`, were created specifically to fill this gap. `mock-happy` writes dummy spec or implementation content and calls `apm state <id> <success-target>` before exiting; `mock-sad` calls `apm state <id> <non-success-target>`. Neither is used in the integration tests. Additionally, a helper `make_mock_worker` exists in the test file as dead code (never called), and `APM_SKIP_COMPAT_CHECK=1` is set in CI to suppress a compat check that is irrelevant once `debug/` is replaced by a named built-in wrapper.

### Acceptance criteria

- [ ] `happy_script` spec-writer mode adds `cd "${APM_PROJECT_ROOT:?}"` before the first `apm` call so all CLI invocations use the main project root
- [ ] `happy_script` impl mode adds `cd "${APM_PROJECT_ROOT:?}"` before the `apm state` call and uses `git -c commit.gpgsign=false commit`
- [ ] Both modes of `happy_script` call `apm state … --force --no-aggressive` instead of bare `apm state`
- [ ] `make_mock_worker` is removed from `apm/tests/integration.rs`
- [x] `APM_SKIP_COMPAT_CHECK: "1"` is removed from the test step in `.github/workflows/release.yml`
- [ ] `setup_with_local_worktrees` patches the agent to `mock-happy/` in both `config.toml` and `workflow.toml`
- [ ] `setup_for_prompt_dispatch` patches the agent to `mock-happy/` in both `config.toml` and `workflow.toml`
- [x] A `wait_for_pid(pid: u32)` helper is added to `integration.rs` that polls `kill -0 <pid>` until the process exits
- [ ] Spawn tests call `apm_core::start::run` (or `apm_core::start::run_next`) directly, extract `worker_pid`, wait via `wait_for_pid`, and assert the final state: `specd` for spec-writer paths and `implemented` for worker paths
- [ ] `cargo test --workspace` passes with all tests green and without `APM_SKIP_COMPAT_CHECK` set

### Out of scope

- mock-sad and mock-random integration test coverage (separate ticket)
- Changes to apm state, apm spec, or other CLI commands beyond what the scripts call
- New integration tests beyond the spawn tests already present
- Fixing repo_root() to resolve main worktree root (the cd fix in happy_script handles the worktree path issue without touching main.rs)

### Approach

#### Fix mock-happy scripts to work from a linked worktree

`mock-happy` runs as a subprocess inside a git linked worktree (e.g. `.apm--worktrees/ticket-foo/`). When its script calls `"$APM" spec …` or `"$APM" state …`, the `apm` binary uses `git rev-parse --show-toplevel` to find the project root, which returns the linked worktree path — not the main repo root. `Config::load` then fails because `.apm/config.toml` does not exist inside the linked worktree.

Fix: in `apm-core/src/wrapper/builtin/mod.rs`, update both branches of `happy_script()`:

- **Spec-writer mode** (the `else` branch): add `cd "${APM_PROJECT_ROOT:?APM_PROJECT_ROOT not set}"` as the first command after the `ID=…` assignment, so all subsequent `apm` calls run from the main project root.
- **Impl mode** (the `if impl_mode` branch): keep the `git add` / `git commit` in the original worktree CWD, then add `cd "${APM_PROJECT_ROOT:?}"` immediately before the `"$APM" state …` call. Also replace the bare `git commit` with `git -c commit.gpgsign=false commit` to avoid failures on systems with global GPG signing enabled.

In both modes, append `--force --no-aggressive` to the `"$APM" state "$ID" {target}` call. `--force` bypasses the completion strategy (no push, no PR creation), making the script work in test repos that have no git remote. `--no-aggressive` suppresses the remote fetch that would also fail without a remote.

`APM_PROJECT_ROOT` is already set to `ctx.root` by `write_and_spawn_script` in the same file; no new env vars are needed.

#### Patch test setup functions

In `apm/tests/integration.rs`, update both `setup_with_local_worktrees()` and `setup_for_prompt_dispatch()`:

1. In the `config.toml` patch, replace `"claude/worker"` with `"mock-happy/worker"`.
2. In the `workflow.toml` patch, replace `"claude/` with `"mock-happy/` (existing replace-all covers all `worker_profile` occurrences).

`setup_with_local_worktrees` already commits the patched files via `git add / git commit`; no change to that commit step is needed. `setup_for_prompt_dispatch` intentionally does not commit (the test reads config from the filesystem); this remains unchanged.

#### Remove dead code and CI env var

- Delete `make_mock_worker` (lines 50–66 in `integration.rs`). It is never called.
- In `.github/workflows/release.yml`, remove the `APM_SKIP_COMPAT_CHECK: "1"` env entry from the test step. `should_check_claude_compat` only fires when the resolved agent is `"claude"` (the built-in Claude wrapper). With `mock-happy` as the agent, the check is never reached.

#### Add `wait_for_pid` helper and refactor spawn tests

Add a helper to `integration.rs`:

```rust
fn wait_for_pid(pid: u32) {
    loop {
        let running = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !running { break; }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
```

`kill -0` checks whether the process exists without sending a signal; it returns non-zero once the PID has been reaped by the background thread in `start::run`.

For each spawn test that currently calls `apm::cmd::start::run(…)` or `apm::cmd::start::run_next(…)` and then immediately asserts ticket state, make the following changes:

1. Call `apm_core::start::run(…)` (or `apm_core::start::run_next(…)`) directly to get `StartOutput` (or `RunNextOutput`) which carries `worker_pid`.
2. Unwrap `worker_pid`, call `wait_for_pid(pid)`.
3. Update the state assertion to the final state that mock-happy produces:
   - Spec-writer path (`groomed`/`ammend` → `in_design`): final state is `specd`.
   - Worker path (`ready` → `in_progress`): final state is `implemented`.

Affected tests:
- `start_spawn_sets_agent_to_worker_pid` — assert `implemented`
- `start_next_spawn_sets_agent_to_worker_pid` — assert `implemented`
- `spawn_new_ticket_transitions_to_in_design` — assert `specd`
- `spawn_ammend_ticket_transitions_to_in_design` — assert `specd`
- `spawn_ready_ticket_transitions_to_in_progress` — assert `implemented`
- `start_next_spawn_new_ticket_transitions_correctly` — assert `specd`
- `start_next_spawn_ready_ticket_transitions_correctly` — assert `implemented`

The `start_non_spawn_keeps_agent_name` test uses `spawn: false`, so mock-happy is never invoked. No change needed there.

Non-spawn tests that use `setup_with_local_worktrees` (e.g. `start_next_claims_highest_priority_ticket`) are unaffected: they never invoke the worker binary.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T19:07Z | — | new | philippepascal |
| 2026-05-24T19:34Z | new | groomed | philippepascal |
| 2026-05-24T19:34Z | groomed | in_design | philippepascal |
| 2026-05-24T19:52Z | in_design | specd | claude |
| 2026-05-24T20:30Z | specd | ready | philippepascal |
| 2026-05-24T20:34Z | ready | in_progress | philippepascal |