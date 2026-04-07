+++
id = "eec00574"
title = "Remove I/O from apm-core: return data, let CLI print"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/eec00574-remove-i-o-from-apm-core-return-data-let"
created_at = "2026-04-07T22:31:00.075025Z"
updated_at = "2026-04-07T22:31:00.075025Z"
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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:31Z | — | new | philippepascal |