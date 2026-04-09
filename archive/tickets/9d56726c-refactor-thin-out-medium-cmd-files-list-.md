+++
id = "9d56726c"
title = "refactor: thin out medium cmd files (list, set, take, workers, worktrees, work)"
state = "closed"
priority = 0
effort = 5
risk = 2
author = "claude-0330-0245-main"
agent = "85993"
branch = "ticket/9d56726c-refactor-thin-out-medium-cmd-files-list-"
created_at = "2026-03-30T14:27:53.108961Z"
updated_at = "2026-03-30T18:08:49.861656Z"
+++

## Spec

### Problem

Several medium-sized CLI command files contain filtering, mutation, or process
monitoring logic that should live in `apm-core`:

**list.rs (40 lines):** Ticket filtering logic (terminal state, actionable state,
agent/supervisor filters) duplicated from other commands. Should call a shared
`apm_core::ticket::list_filtered()`.

**set.rs (65 lines):** Field validation (priority/effort/risk range checks) and
immutability enforcement (author field) belong in `apm-core::ticket::set_field()`.

**take.rs (78 lines):** Agent handoff validation and history append belong in
`apm_core::ticket::handoff()`.

**workers.rs (277 lines):** PID file parsing, process liveness checks (`kill -0`),
elapsed time calculation, and kill logic are business logic. Only the table
formatting belongs in the CLI.

**worktrees.rs (67 lines):** Worktree enumeration with branch-to-ticket matching
should delegate to `apm_core::git::list_worktrees_with_tickets()`.

**work.rs (93 lines):** Worker pool management and result state validation should
delegate more cleanly to `apm-core` rather than calling across cmd modules.

Individually small, but collectively ~600 lines of leaked business logic that
blocks `apm-serve` from reusing any of it.

### Acceptance criteria

- [x] `apm list` with no flags produces identical output before and after the refactor
- [x] `apm list --state <s>`, `--unassigned`, `--all`, `--supervisor`, `--actionable` flags each produce identical output
- [x] `apm set <id> author foo` still returns error "author is immutable"
- [x] `apm set <id> priority 256` still returns a parse error (u8 overflow)
- [x] `apm set <id> unknownfield val` still returns error "unknown field: unknownfield"
- [x] `apm take <id>` on a ticket with no agent still returns "no agent assigned — use `apm start` instead"
- [x] `apm workers list` shows the same columns (ID, TITLE, PID, STATE, ELAPSED) with identical process monitoring behavior
- [x] `apm worktrees` produces identical output
- [x] `apm work --dry-run` produces identical output
- [x] `ticket::list_filtered()` exists in apm-core and is unit-tested for state, terminal-visibility, agent, supervisor, and actionable-actor filtering
- [x] `ticket::set_field()` exists in apm-core and is unit-tested for valid mutations and all error cases (author immutability, invalid u8, unknown field)
- [x] `ticket::handoff()` exists in apm-core and is unit-tested: rejects missing agent, is idempotent when already assigned, and appends a history row on successful transfer
- [x] apm-core exposes a `worker` module with `PidFile`, `read_pid_file()`, `is_alive()`, and `elapsed_since()`, each unit-tested
- [x] `ticket::list_worktrees_with_tickets()` exists in apm-core and is called by `cmd/worktrees.rs`
- [x] `cargo test --workspace` passes with no regressions

### Out of scope

- Changing any CLI output format or interface
- Moving logic from cmd files not listed in the ticket (new.rs, state.rs, spec.rs, start.rs, show.rs, etc.)
- Implementing `apm-serve` or any new binary
- Adding new filtering or validation capabilities
- Changing config schema or apm.toml format
- Performance optimizations

### Approach

**1. `ticket::list_filtered()` — apm-core/src/ticket.rs**

Add a pub function with this signature:
```rust
pub fn list_filtered<'a>(
    tickets: &'a [Ticket],
    config: &Config,
    state_filter: Option<&str>,
    unassigned: bool,
    all: bool,
    supervisor_filter: Option<&str>,
    actionable_filter: Option<&str>,
) -> Vec<&'a Ticket>
```
Move the terminal-state HashSet, actionable-actor HashMap, and five boolean filter predicates verbatim from `cmd/list.rs` into this function. `cmd/list.rs` becomes a thin loop: call `list_filtered()`, then print each result. Add a unit test in ticket.rs that covers each filter axis.

**2. `ticket::set_field()` — apm-core/src/ticket.rs**

Add a pub function:
```rust
pub fn set_field(fm: &mut Frontmatter, field: &str, value: &str) -> Result<()>
```
Move the `match field` arms from `cmd/set.rs` into this function. It validates and mutates the Frontmatter only — it does not touch `updated_at` (that stays in cmd, set after the call). `cmd/set.rs` shrinks to: resolve id, call `set_field()`, set `updated_at`, serialize, commit. Add unit tests for every branch.

**3. `ticket::handoff()` — apm-core/src/ticket.rs**

Add a pub function:
```rust
pub fn handoff(ticket: &mut Ticket, new_agent: &str, now: DateTime<Utc>) -> Result<Option<String>>
```
Returns `Ok(None)` when already assigned to `new_agent`; `Ok(Some(old_agent))` on successful transfer; `Err` when no agent is currently assigned. Implementation:
- Guard: `fm.agent.is_none()` → bail with the existing error message
- Short-circuit: `old == new_agent` → return Ok(None)
- Mutate: set `fm.agent`, `fm.updated_at`
- Append a history row: copy the `append_history` string-manipulation logic from `cmd/state.rs` into ticket.rs as a module-private helper `fn append_history_row(body: &mut String, from: &str, to: &str, when: &str, by: &str)`. `cmd/state.rs` can call the same helper via re-export, or keep its own copy if it needs to diverge.

`cmd/take.rs` shrinks to: read APM_AGENT_NAME, resolve id, call `handoff()`, serialize, commit, provision worktree. Add three unit tests: no-agent error, idempotent, and successful transfer (check history row appended).

**4. `apm-core/src/worker.rs` — new module**

Create this file and expose:
```rust
pub struct PidFile {
    pub ticket_id: String,
    pub started_at: String,
}
pub fn read_pid_file(path: &Path) -> Result<(u32, PidFile)>
pub fn is_alive(pid: u32) -> bool
pub fn elapsed_since(started_at: &str) -> String
```
Move these from `cmd/workers.rs` verbatim. `cmd/workers.rs` imports them from `apm_core::worker`. Declare the module in `apm-core/src/lib.rs`. Add unit tests for `elapsed_since` (seconds, minutes, hours, unparseable input) and `read_pid_file` (valid JSON, missing fields). Kill logic in `cmd/workers.rs` can call `is_alive()` from core but otherwise stays in the cmd layer because it issues OS signals, not business logic.

**5. `ticket::list_worktrees_with_tickets()` — apm-core/src/ticket.rs**

Add a pub function (placed in ticket.rs rather than git.rs to avoid a circular dependency, since ticket.rs already imports git):
```rust
pub fn list_worktrees_with_tickets(
    root: &Path,
    tickets_dir: &Path,
) -> Result<Vec<(PathBuf, String, Option<Ticket>)>>
```
Calls `git::list_ticket_worktrees(root)?`, loads tickets, then matches each `(wt_path, branch)` to a ticket by `frontmatter.branch` or `branch_name_from_path`. Returns `(wt_path, branch, Option<Ticket>)` triples. `cmd/worktrees.rs` replaces its inline loop with a call to this function.

**6. work.rs — fix hardcoded states**

The worker-pool spawn loop and the result-state check both stay in cmd — they cannot move to apm-core without also moving the spawn logic. However, the hardcoded `["implemented", "specd"]` slice must be replaced with a config-derived set:

```rust
// Instead of: let good_states = ["implemented", "specd"];
let good_states: Vec<&str> = config.workflow.states.iter()
    .filter(|s| s.terminal)
    .map(|s| s.id.as_str())
    .collect();
```

`StateConfig` already has `terminal: bool` — no schema change needed. Nothing moves to apm-core for this step; the fix is entirely within `cmd/work.rs`.

**Order of changes**
1. Add `list_filtered()` + tests → update list.rs
2. Add `set_field()` + tests → update set.rs
3. Add `handoff()` + append_history_row helper + tests → update take.rs
4. Create worker.rs + tests → update workers.rs imports
5. Add `list_worktrees_with_tickets()` + tests → update worktrees.rs
6. Fix hardcoded states in work.rs using `config.workflow.states`
7. Run `cargo test --workspace`; confirm all green

### Open questions



### Amendment requests

- [x] 6. work.rs: we don't want to hard code states that are defined in configuration. If this can't be done, maybe some part of work.rs can't move to apm-core.


### Code review
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:36Z | new | in_design | philippepascal |
| 2026-03-30T16:41Z | in_design | specd | claude-0330-1640-sp01 |
| 2026-03-30T16:50Z | specd | ammend | philippepascal |
| 2026-03-30T17:29Z | ammend | in_design | philippepascal |
| 2026-03-30T17:32Z | in_design | specd | claude-0330-1800-sp02 |
| 2026-03-30T17:46Z | specd | ready | philippepascal |
| 2026-03-30T17:46Z | ready | in_progress | philippepascal |
| 2026-03-30T17:55Z | in_progress | implemented | claude-0330-1800-wk01 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |