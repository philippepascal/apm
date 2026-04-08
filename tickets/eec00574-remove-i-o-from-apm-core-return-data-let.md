+++
id = "eec00574"
title = "Remove I/O from apm-core: return data, let CLI print"
state = "closed"
priority = 0
effort = 6
risk = 3
author = "philippepascal"
branch = "ticket/eec00574-remove-i-o-from-apm-core-return-data-let"
created_at = "2026-04-07T22:31:00.075025Z"
updated_at = "2026-04-08T04:03:06.290144Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
depends_on = ["eea2c9bc", "a71186da", "24069bd8", "fe6e9d1d", "ce919ea8"]
+++

## Spec

### Problem

The `apm-core` library crate contains direct I/O operations (`println!`, `eprintln!`, `stdin().read_line()`) in at least 8 modules:

- `init.rs` — interactive prompts for username, project name, project description via stdin/stdout
- `state.rs` — `eprintln!` for merge conflict warnings, `println!` for state change confirmation
- `start.rs` — `println!` for worker spawn status, error messages
- `archive.rs` — `println!` for each archived ticket, `eprintln!` for warnings
- `config.rs` — `eprintln!` in identity resolution and collaborator fetching
- `sync.rs` — `eprintln!` in sync apply logic
- `git.rs` — `println!` in PR creation and merge output
- `clean.rs` — `eprintln!` warnings during candidate detection

The intended architecture is that `apm-core` is a pure logic library and `apm` (the binary crate) handles all user-facing I/O. The current I/O bleed makes apm-core impossible to use as a library (e.g., from a web server or test harness) without unwanted output to stdout/stderr. It also makes testing harder — tests can't capture or assert on output that goes directly to stdio.

The fix is to have apm-core functions return structured results (data, warnings, errors) and let the CLI layer in `apm/src/cmd/` decide how to present them.

### Acceptance criteria

- [x] `apm-core` compiles with `#![deny(clippy::print_stdout, clippy::print_stderr)]` without errors — confirming zero println!/eprintln! remain in the library crate
- [x] `apm init` produces identical user-visible output to today's behaviour after moving all println!/stdin logic to `apm/src/cmd/init.rs`
- [x] `apm init --with-docker` prints Docker setup instructions via the CLI handler, not from within `apm_core::init::setup_docker`
- [x] `apm state <id> <state>` continues to print `<id>: <old> → <new>` and the worktree path, driven by the CLI handler printing fields from `TransitionOutput`
- [x] `apm state` warnings (push failures, merge conflict notices) are printed by the CLI handler using a `warnings: Vec<String>` field on `TransitionOutput`
- [x] `apm start <id>` continues to print worker spawn status and branch info, driven by `StartOutput` fields printed in `apm/src/cmd/start.rs`
- [x] `apm start --next` continues to print "No actionable tickets." or spawn info, now returned via a `RunNextOutput` struct instead of printed inside `apm_core::start::run_next`
- [x] `apm archive` prints archived file moves and a final count, driven by an `ArchiveOutput` struct with `moves: Vec<(String, String)>`, `archived_count: usize`, and `warnings: Vec<String>` fields
- [x] `apm sync` apply warnings (failed closures) are surfaced to the caller via an `ApplyOutput` struct rather than eprintln in `apm_core::sync::apply`
- [x] `apm clean` branch-deletion warnings are returned as collected strings in the `remove` return value rather than printed inside `apm_core::clean::remove`
- [x] All existing integration/unit tests pass after the refactor
- [x] No existing public API symbols are removed — only return types are widened (existing struct fields are preserved; new fields are additive)

### Out of scope

- Changing the text/wording of any user-facing messages (this ticket only moves where they are printed, not what they say)
- Adding structured logging, tracing, or a logging framework to apm-core
- Removing or changing the interactive prompts in `apm init` beyond moving them from core to the CLI layer
- apm-server or any web-facing consumer of apm-core (no server-side I/O handlers are written here)
- Adding new commands or changing command behaviour
- Fixing unrelated bugs discovered during the refactor (note them, don't fix them)

### Approach

The existing pattern — `TransitionOutput` returned by `state::transition`, printed by `apm/src/cmd/state.rs` — is the reference. Every change here follows that exact pattern.

Add `#![deny(clippy::print_stdout, clippy::print_stderr)]` to `apm-core/src/lib.rs` first. The compiler errors that result are the work list. Fix them module by module.

---

**apm-core/src/init.rs**

The `setup()` function calls several internal helpers (`write_default`, `ensure_gitignore`, `ensure_claude_md`, `maybe_initial_commit`, `ensure_worktrees_dir`) that each println the action they took. Thread a `messages: &mut Vec<String>` parameter through all of them; replace every `println!` call with `messages.push(format!(...))`. Return `SetupOutput { messages: Vec<String> }` from `setup()`.

For `setup_docker()`, return `SetupDockerOutput { messages: Vec<String> }` the same way.

The interactive prompt helpers (`prompt_project_info`, `prompt_username`) use stdin/stdout. Move their bodies into `apm/src/cmd/init.rs` — the CLI handler already calls `setup()`, so the prompts run first in the CLI, collect the strings, and pass them as parameters (e.g. `setup(root, name: Option<&str>, description: Option<&str>, username: Option<&str>)`). When any `Option` is `None`, the core function uses a sensible default or derives the value non-interactively (e.g. from git config).

Update `apm/src/cmd/init.rs` to: run prompts → call `apm_core::init::setup(...)` → print `out.messages`.

---

**apm-core/src/state.rs**

`TransitionOutput` already exists. Add two fields:
```rust
pub warnings: Vec<String>,
pub messages: Vec<String>,
```
Replace every `eprintln!` in `transition`, `pull_default`, `push_and_sync_refs`, and `gh_pr_create_or_update` with `warnings.push(...)`. Replace every `println!` (PR URL, merge confirmation) with `messages.push(...)`. Propagate the vectors through each internal call site.

Update `apm/src/cmd/state.rs` to print `out.warnings` to stderr and `out.messages` to stdout after the existing `id: old → new` line.

---

**apm-core/src/start.rs**

`run()` already returns `StartOutput`; add `warnings: Vec<String>` to it and replace its `eprintln!` calls. No CLI change needed for `run()` beyond printing new warnings.

`run_next()` currently returns `Result<()>` and prints directly. Give it a return type:
```rust
pub struct RunNextOutput {
    pub ticket_id: Option<String>,
    pub messages: Vec<String>,
    pub warnings: Vec<String>,
    pub worker_pid: Option<u32>,
    pub log_path: Option<PathBuf>,
}
```
Replace all `println!`/`eprintln!` in `run_next` with pushes into these vectors; return `Ok(RunNextOutput { ... })`.

`spawn_next_worker` is called by `run_next`. Collect its output into the `RunNextOutput` vectors rather than printing.

Update `apm/src/cmd/start.rs::run_next` to print fields from `RunNextOutput`.

---

**apm-core/src/archive.rs**

Change `archive()` signature to return:
```rust
pub struct ArchiveOutput {
    pub moves: Vec<(String, String)>,   // (old_rel_path, new_rel_path)
    pub archived_count: usize,
    pub warnings: Vec<String>,
}
pub fn archive(...) -> Result<ArchiveOutput>
```
Collect moves in a `Vec` (already done for the dry-run loop), replace `println!("nothing to archive")` and `println!("archived {} ticket(s)")` with data in the struct. Collect `eprintln!` calls into `warnings`.

Update `apm/src/cmd/archive.rs` to print the moves, count, and warnings.

---

**apm-core/src/sync.rs**

`apply()` currently returns `Result<()>`. Change to:
```rust
pub struct ApplyOutput {
    pub closed: Vec<String>,            // ids successfully closed
    pub failed: Vec<(String, String)>,  // (id, error message)
}
pub fn apply(...) -> Result<ApplyOutput>
```
Replace `eprintln!("warning: could not close ...")` with a push to `failed`.

Update `apm/src/cmd/sync.rs` to print failed closures as warnings.

---

**apm-core/src/git.rs**

All I/O here is `eprintln!` warnings in helper functions (`sync_agent_dirs`, `push_and_sync_refs`, `sync_local_ticket_refs`, `merge_default_branch`). These helpers are called by `state::transition` and `start::run`, which now carry `warnings: Vec<String>`. Pass `&mut Vec<String>` down to the git helpers so they push into the caller's warning list. No new public structs needed; the existing `TransitionOutput.warnings` and `StartOutput.warnings` absorb them.

---

**apm-core/src/clean.rs**

`remove()` currently returns `Result<()>`. Change to:
```rust
pub struct RemoveOutput {
    pub warnings: Vec<String>,
}
pub fn remove(...) -> Result<RemoveOutput>
```
Replace the two branch-deletion `eprintln!` calls with pushes to `warnings`.

`candidates()` has one `eprintln!`. Add a `warnings: Vec<String>` field to its existing return tuple or wrap in a struct:
```rust
pub struct CandidatesOutput {
    pub candidates: Vec<CleanCandidate>,
    pub dirty: Vec<DirtyWorktree>,
    pub warnings: Vec<String>,
}
```

Update `apm/src/cmd/clean.rs` to print the warnings.

---

**apm-core/src/config.rs**

Two `eprintln!` calls remain:

1. `resolve_identity` (line 399): the `eprintln!("apm: could not resolve identity from git_host")` fires just before returning the fallback `"unassigned"`. Remove the `eprintln!` entirely — the "unassigned" return value already signals the failure to callers; the warning adds no actionable information and no call site currently checks for it. No signature change required.

2. `resolve_collaborators` (line 430): currently returns `Vec<String>`. Change to return a tuple `(Vec<String>, Vec<String>)` — `(collaborators, warnings)`. Replace the `eprintln!` with a push to the warnings vec. Update all call sites (search for `resolve_collaborators` in `apm-core` and `apm`) to destructure the tuple; print the warning strings to stderr in the CLI handler.

---

**Order of changes**

1. Add the deny lint to `lib.rs` — this surfaces all sites at once
2. Fix `git.rs` helpers first (they are called by state/start; fixing them unblocks those)
3. Fix `state.rs`, `start.rs` (high call-graph, touch most warnings)
4. Fix `archive.rs`, `sync.rs`, `clean.rs` (isolated, straightforward)
5. Fix `config.rs` (two minor calls; no new structs for `resolve_identity`)
6. Fix `init.rs` last (most involved due to stdin prompts)
7. Update all CLI handlers in `apm/src/cmd/`
8. Confirm `cargo test` passes and the deny lint has no violations

### Open questions


### Amendment requests

- [x] The spec excludes `config.rs` eprintln calls ("can be addressed in a follow-on cleanup ticket"), but the proposed `#![deny(clippy::print_stdout, clippy::print_stderr)]` lint will fail to compile if those calls remain. Either: (a) include `config.rs` in scope and remove it from the Out of scope section, or (b) change the approach to use `#![warn(...)]` initially and note that a follow-on ticket upgrades to `deny` after `config.rs` is cleaned up. Pick whichever is simpler.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:31Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T23:03Z | groomed | in_design | philippepascal |
| 2026-04-07T23:07Z | in_design | specd | claude-0407-2303-b708 |
| 2026-04-08T00:03Z | specd | ammend | philippepascal |
| 2026-04-08T00:37Z | ammend | in_design | philippepascal |
| 2026-04-08T00:40Z | in_design | specd | claude-0408-0037-2de0 |
| 2026-04-08T01:08Z | specd | ready | apm |
| 2026-04-08T01:09Z | ready | in_progress | philippepascal |
| 2026-04-08T01:42Z | in_progress | implemented | claude-0408-0109-5aa0 |
| 2026-04-08T04:03Z | implemented | closed | apm-sync |
