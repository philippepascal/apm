+++
id = 2
title = "Add integration tests for CLI commands"
state = "in_progress"
priority = 10
effort = 3
risk = 2
agent = "claude-0325-2043-a970"
branch = "feature/2-integration-tests"
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Amendment requests
- [x] specify where the temp git repo will be. I imagine it can't be within the project repo. we probably need a setting somewhere were the user can setup a test-sandbox dir

  Addressed: `tempfile::tempdir()` creates directories under the OS temp dir (`/tmp/`
  on macOS), entirely outside the project repo. No config setting is needed. Because
  commands accept an explicit `root: &Path` parameter (see Approach), `git rev-parse`
  is never called during tests — there is no risk of tests running inside the APM repo.

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
| 2026-03-25 | manual | specd → ammend | |
| 2026-03-25 | manual | ammend → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-25 | manual | ready → in_progress | |
| 2026-03-25 | manual | in_progress → in_progress | |
