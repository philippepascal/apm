+++
id = 2
title = "Add integration tests for CLI commands"
state = "specd"
priority = 10
effort = 3
risk = 2
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

The CLI commands (`init`, `new`, `list`, `show`, `state`, `set`, `next`) have no
integration tests. Unit tests cover parsing logic but not the full command path:
config loading, file I/O, git detection, and output formatting. Bugs in command
wiring are invisible until manual testing.

### Acceptance criteria

- [ ] `apm init` creates `tickets/`, `apm.toml`, `.gitignore` in a temp git repo
- [ ] `apm init` is idempotent (running twice does not overwrite existing files)
- [ ] `apm new "title"` creates a correctly named and formatted ticket file
- [ ] `apm list` prints all tickets; `--state` filters correctly
- [ ] `apm show <id>` prints ticket fields and body
- [ ] `apm state <id> <state>` updates the file and appends a history row
- [ ] `apm set <id> priority <n>` updates the frontmatter field
- [ ] `apm next` returns the highest-scoring unassigned actionable ticket
- [ ] `apm next --json` returns valid JSON with id, title, state, score
- [ ] All tests run with `cargo test --workspace` without manual setup

### Out of scope

- Testing `apm sync`, `apm start`, `apm take` (not yet implemented)
- Provider/GitHub integration
- SQLite cache (not yet implemented)

### Approach

`tests/` directory in the `apm` crate. Each test:
1. Creates a `tempdir` with `git init`
2. Invokes the command function directly (not via subprocess) by calling `cmd::*::run()`
   with the working directory set via `std::env::set_current_dir` or by threading the
   root path through — prefer passing root explicitly to avoid test parallelism issues
3. Asserts on file contents and stdout where applicable

The root path threading approach requires refactoring each `cmd::*::run()` to accept
an explicit `root: &Path` parameter instead of calling `repo_root()` internally.
This makes commands testable without a real git process and without `set_current_dir`
races between parallel tests.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
