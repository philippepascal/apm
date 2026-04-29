+++
id = "bc89e0a0"
title = "Add apm help command with git-style topic dispatch"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc89e0a0-add-apm-help-command-with-git-style-topi"
created_at = "2026-04-28T19:27:00.760945Z"
updated_at = "2026-04-29T06:55:25.917973Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
+++

## Spec

### Problem

There is no unified `apm help` command today. Users discover apm surface area by running `apm <subcommand> --help` for each command individually and reading source for config schemas. A git-style `apm help [topic]` entry point would give users a single landing point to orient themselves across commands, config, and workflow concepts.

This ticket adds CLI plumbing only: the `Help` subcommand variant in the clap `Command` enum, dispatch wiring in `main()`, and a new `cmd::help` module with four stub renderer functions. No real content is produced here; topic content arrives in sibling tickets within this epic.

### Acceptance criteria

- [x] `apm help` (no topic) exits 0 and prints a short description of the help system
- [x] `apm help` (no topic) lists all available topics (`commands`, `config`, `workflow`, `ticket`) with a one-line summary each
- [x] `apm help commands` exits 0 and prints a non-empty placeholder string referencing ticket 3665e017
- [x] `apm help config` exits 0 and prints a non-empty placeholder string referencing ticket d486d183
- [ ] `apm help workflow` exits 0 and prints a non-empty placeholder string referencing ticket 7ba021e8
- [ ] `apm help ticket` exits 0 and prints a non-empty placeholder string referencing ticket 14214305
- [ ] `apm help <unknown-topic>` exits non-zero
- [ ] `apm help <unknown-topic>` prints an error message that names the unknown topic and lists the valid topics
- [ ] `apm --help` lists `help` as a subcommand in the clap-generated usage output

### Out of scope

- Actual content for any topic (`commands`, `config`, `workflow`, `ticket`) — each is a sibling ticket in this epic
- Auto-derive infrastructure for rendering TOML schemas from Rust structs (ticket 069c3403)
- Pager integration, markdown rendering, or color/ANSI output
- Fuzzy-matching or "did you mean?" suggestions on unknown topics
- Any changes to how `apm <subcommand> --help` works (clap-native help is untouched)

### Approach

**Files to change:**

**1. apm/src/cmd/help.rs (new file)**

- Define a private TOPICS: &[(&str, &str)] constant listing (commands, one-liner), (config, one-liner), (workflow, one-liner), (ticket, one-liner). One source of truth used by both the overview and the error path.
- pub fn run(topic: Option<&str>) -> Result<()>:
  - None -> print render_overview() to stdout, exit 0
  - Some(t) -> match t against TOPICS names; dispatch to the matching render_*() fn and print; unknown -> anyhow::bail! with the topic name and a list of valid topics from TOPICS
- fn render_overview() -> String: short description paragraph + table of TOPICS (name padded, one-line summary)
- fn render_commands() -> String: returns placeholder string referencing ticket 3665e017
- fn render_config() -> String: returns placeholder string referencing ticket d486d183
- fn render_workflow() -> String: returns placeholder string referencing ticket 7ba021e8
- fn render_ticket() -> String: returns placeholder string referencing ticket 14214305
- run() does NOT take root: &Path -- the help command needs no repo context.

**2. apm/src/main.rs**

- Add Help { topic: Option<String> } to the Command enum with a doc comment.
- Add Command::Help { topic } => cmd::help::run(topic.as_deref())? to the match dispatch.

**3. apm/src/lib.rs (or wherever cmd submodules are declared)**

- Add pub mod help; inside the pub mod cmd { ... } block alongside the other 27 command modules.

**Implementation order:**
1. Create apm/src/cmd/help.rs with TOPICS constant, run(), render_overview(), and four stubs
2. Register pub mod help; in the cmd block
3. Add Help variant and dispatch arm to main.rs
4. cargo build to confirm the module compiles and the match is exhaustive
5. Smoke-test all nine acceptance criteria cases manually

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:33Z | groomed | in_design | philippepascal |
| 2026-04-28T19:37Z | in_design | specd | claude-0428-1933-feb0 |
| 2026-04-29T03:42Z | specd | ready | philippepascal |
| 2026-04-29T03:43Z | ready | in_progress | philippepascal |
| 2026-04-29T03:47Z | in_progress | ready | philippepascal |
| 2026-04-29T06:55Z | ready | in_progress | philippepascal |