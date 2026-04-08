+++
id = "31429c7d"
title = "refactor: move state machine logic from state.rs and start.rs into apm-core"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "claude-0330-0245-main"
agent = "95554"
branch = "ticket/31429c7d-refactor-move-state-machine-logic-from-s"
created_at = "2026-03-30T14:27:29.706701Z"
updated_at = "2026-03-30T18:08:07.568579Z"
+++

## Spec

### Problem

`state.rs` and `start.rs` together contain ~780 lines of business logic that
belongs in `apm-core`:

**state.rs (274 lines):**
- State transition validation against config (allowed transitions, actor checks)
- Document validation before transition (spec sections required for "specd",
  AC checked for "implemented", amendment boxes for "ammend")
- History entry appending (`append_history`)
- Amendment section auto-creation
- PR creation via `gh` CLI (completion strategy logic)
- Merge into default branch with conflict handling
- Worktree provisioning for `in_design` state

**start.rs (509 lines):**
- Startable state detection from config
- State machine transition execution
- Worktree provisioning and merge-from-default
- Worker system prompt loading (`.apm/worker.md` fallback)
- PID file writing and cleanup thread
- Focus section extraction and clearing

These two files are tightly coupled — `start.rs` calls `state.rs` functions and
both manipulate the same state machine. Neither belongs in the CLI layer. `apm-serve`
will need to perform state transitions and start workers without shelling out to
the CLI or duplicating logic.

Target: `apm_core::state::transition()` and `apm_core::start::run()` with thin
CLI wrappers of ~30 lines each.

### Acceptance criteria

- [x] `apm_core::state` module exists and exports a `transition()` function encapsulating all transition logic currently in `cmd/state::run()`
- [x] `apm_core::start` module exists and exports `run()`, `run_next()`, and `spawn_next_worker()` containing all start logic currently in `cmd/start.rs`
- [x] `append_history()` and `ensure_amendment_section()` are public functions in `apm_core::state` (no longer in the CLI layer)
- [x] `resolve_agent_name()` is a public function in `apm_core::start` (no longer in the CLI layer)
- [x] `apm/src/cmd/state.rs` contains no business logic — it parses CLI args and delegates entirely to `apm_core::state::transition()`
- [x] `apm/src/cmd/start.rs` contains no business logic — it parses CLI args and delegates entirely to the corresponding `apm_core::start` functions
- [x] `apm state <id> <new_state>` produces identical output to before the refactor
- [x] `apm start <id>` produces identical output to before the refactor
- [x] `apm start --next --spawn` produces identical output to before the refactor
- [x] `cargo test --workspace` passes after the refactor

### Out of scope

- `apm-serve` integration or any new callers of the extracted core functions
- Changes to `apm_core::git` or `apm_core::ticket` beyond what is needed to support the new modules
- Changes to any other CLI commands (`apm new`, `apm list`, etc.)
- Performance improvements or behaviour changes — this is a pure mechanical extraction

### Approach

**1. Create `apm-core/src/state.rs`**

Move the following from `apm/src/cmd/state.rs`:
- `pub fn transition(root, id, new_state, force, no_aggressive) -> Result<TransitionOutput>` — renamed from `run()`, returns a struct instead of printing directly. `TransitionOutput` holds `id`, `old_state`, `new_state`, and an optional worktree path (for `in_design` transitions).
- `pub fn append_history(body, from, to, when, by)` — unchanged signature, moved verbatim
- `pub fn ensure_amendment_section(body)` — unchanged signature, moved verbatim
- `fn gh_pr_create_or_update(root, branch, default_branch, id, title)` — private to the module, moved verbatim
- `fn merge_into_default(root, branch, default_branch)` — private to the module, moved verbatim

`transition()` returns `TransitionOutput` and does not call `println!`. The CLI wrapper in `cmd/state.rs` prints the result.

**2. Create `apm-core/src/start.rs`**

Move the following from `apm/src/cmd/start.rs`:
- `pub fn run(root, id, no_aggressive, spawn, skip_permissions, agent_name) -> Result<StartOutput>` — unchanged logic, returns a struct with fields the CLI needs to print. `StartOutput` holds `id`, `old_state`, `new_state`, `agent_name`, `branch`, `worktree_path`, and optionally `worker_pid` and `log_path`.
- `pub fn run_next(root, no_aggressive, spawn, skip_permissions) -> Result<()>` — unchanged logic (may still print internally since it is more of an orchestrator; refine to return output struct if needed)
- `pub fn spawn_next_worker(root, no_aggressive, skip_permissions) -> Result<Option<(String, Child, PathBuf)>>` — unchanged signature, moved verbatim
- `pub fn resolve_agent_name() -> String` — moved verbatim
- `fn write_pid_file(path, pid, ticket_id)` — private, moved verbatim
- `fn rand_u16() -> u16` — private, moved verbatim

**3. Update `apm-core/src/lib.rs`**

Add:
```rust
pub mod state;
pub mod start;
```

**4. Rewrite `apm/src/cmd/state.rs`**

~25 lines: parse args, call `apm_core::state::transition()`, print `TransitionOutput`.

**5. Rewrite `apm/src/cmd/start.rs`**

~25 lines each for `run`, `run_next`, `spawn_next_worker`: delegate to `apm_core::start`.

**6. Fix import in `start.rs`**

`start.rs` currently calls `super::state::append_history()`. After the move it calls `apm_core::state::append_history()`.

**Order of steps:**
1. Create `apm-core/src/state.rs` and `apm-core/src/start.rs` with the moved logic
2. Add modules to `apm-core/src/lib.rs`
3. Rewrite the thin CLI wrappers
4. Run `cargo test --workspace`

**Known constraints:**
- `rand_u16()` uses `SystemTime` — no external dependency needed
- Worker spawning shells out to `claude` — stays in `apm_core::start` since `apm-serve` also needs to spawn workers
- The existing unit tests in `cmd/start.rs` (`resolve_agent_name` tests) move to `apm-core/src/start.rs`

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
| 2026-03-30T16:35Z | in_design | specd | claude-0330-1640-b7e2 |
| 2026-03-30T16:56Z | specd | ready | philippepascal |
| 2026-03-30T17:29Z | ready | in_progress | philippepascal |
| 2026-03-30T17:38Z | in_progress | implemented | claude-0330-1800-w4rk |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |