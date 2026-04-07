+++
id = "eec00574"
title = "Remove I/O from apm-core: return data, let CLI print"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/eec00574-remove-i-o-from-apm-core-return-data-let"
created_at = "2026-04-07T22:31:00.075025Z"
updated_at = "2026-04-07T23:03:13.530560Z"
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

- [ ] `apm-core` compiles with `#![deny(clippy::print_stdout, clippy::print_stderr)]` without errors — confirming zero println!/eprintln! remain in the library crate
- [ ] `apm init` produces identical user-visible output to today's behaviour after moving all println!/stdin logic to `apm/src/cmd/init.rs`
- [ ] `apm init --with-docker` prints Docker setup instructions via the CLI handler, not from within `apm_core::init::setup_docker`
- [ ] `apm state <id> <state>` continues to print `<id>: <old> → <new>` and the worktree path, driven by the CLI handler printing fields from `TransitionOutput`
- [ ] `apm state` warnings (push failures, merge conflict notices) are printed by the CLI handler using a `warnings: Vec<String>` field on `TransitionOutput`
- [ ] `apm start <id>` continues to print worker spawn status and branch info, driven by `StartOutput` fields printed in `apm/src/cmd/start.rs`
- [ ] `apm start --next` continues to print "No actionable tickets." or spawn info, now returned via a `RunNextOutput` struct instead of printed inside `apm_core::start::run_next`
- [ ] `apm archive` prints archived file moves and a final count, driven by an `ArchiveOutput` struct with `moves: Vec<(String, String)>`, `archived_count: usize`, and `warnings: Vec<String>` fields
- [ ] `apm sync` apply warnings (failed closures) are surfaced to the caller via an `ApplyOutput` struct rather than eprintln in `apm_core::sync::apply`
- [ ] `apm clean` branch-deletion warnings are returned as collected strings in the `remove` return value rather than printed inside `apm_core::clean::remove`
- [ ] All existing integration/unit tests pass after the refactor
- [ ] No existing public API symbols are removed — only return types are widened (existing struct fields are preserved; new fields are additive)

### Out of scope

- Changing the text/wording of any user-facing messages (this ticket only moves where they are printed, not what they say)
- Adding structured logging, tracing, or a logging framework to apm-core
- Removing or changing the interactive prompts in `apm init` beyond moving them from core to the CLI layer
- apm-server or any web-facing consumer of apm-core (no server-side I/O handlers are written here)
- The `config.rs` identity/collaborator eprintln calls — these are very minor and can be addressed in a follow-on cleanup ticket
- Adding new commands or changing command behaviour
- Fixing unrelated bugs discovered during the refactor (note them, don't fix them)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:31Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T23:03Z | groomed | in_design | philippepascal |