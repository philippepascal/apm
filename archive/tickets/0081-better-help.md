+++
id = "0081"
title = "better help"
state = "closed"
priority = 0
effort = 3
risk = 1
author = "apm"
agent = "claude-0330-0245-main"
branch = "ticket/0081-better-help"
created_at = "2026-03-30T04:47:00.345986Z"
updated_at = "2026-03-30T18:07:33.794341Z"
+++

## Spec

### Problem

Every `apm` command currently has only a single-line description — the `///`
doc comment on its enum variant in `apm/src/main.rs`. When a user runs
`apm <command> --help` they see no more than that one sentence plus argument
names, with no indication of where the command fits in the overall agent
workflow (e.g. "run this before `start`", "supervisor-only", "only valid from
`in_progress`").

Clap 4 distinguishes `-h` (short help, uses `about`) from `--help` (long help,
uses `long_about`). Today every command has identical output for both flags.
Adding `long_about` text to each command lets the short flag stay terse while
`--help` surfaces workflow context, prerequisites, and practical examples —
without changing any runtime behaviour.

### Acceptance criteria

- [x] `apm --help` shows a multi-paragraph description of the tool and the overall workflow (new → specd → ready → in_progress → implemented → closed)
- [x] `apm -h` still shows the current terse one-liner for the tool
- [x] `apm list --help` output is longer than `apm list -h` output
- [x] `apm new --help` describes where `new` fits in the lifecycle and mentions `--no-edit` for agent use
- [x] `apm state --help` explains that valid target states depend on the current state and refers the user to `apm.toml`
- [x] `apm start --help` explains the claim-and-worktree semantics and `--spawn` option
- [x] `apm sync --help` explains what sync does (fetch, detect merges, close stale tickets)
- [x] `apm review --help` explains the supervisor role and the `--to` flag
- [x] `apm worktrees --help` explains permanent worktrees and the `--add`/`--remove` flags
- [x] `apm next --help` explains priority ordering and the `--json` flag for agent use
- [x] `apm take --help` explains the takeover scenario (agent crashed, reassignment)
- [x] `apm work --help` explains the orchestration loop and `--dry-run`
- [x] `apm spec --help` explains the section read/write model and `--mark`
- [x] `apm close --help` clarifies this is a supervisor-only force-close
- [x] `apm clean --help` explains what is removed and the safety of `--dry-run`
- [x] `apm show --help` is longer than the current single-line description
- [x] `apm set --help` lists the valid field names in its long description
- [x] `apm init --help` explains what init creates (`.apm/` directory, `apm.toml`, hooks)
- [x] `apm agents --help` explains what the command prints and when to use it
- [x] `apm validate --help` describes the checks performed and the `--fix` behaviour
- [x] The internal `_hook` command is hidden from the top-level command list (users should not see it)

### Out of scope

- Changing any runtime behaviour of any command
- Restructuring the CLI (adding or removing commands, flags, or arguments)
- Updating `apm.agents.md` or `CLAUDE.md` documentation files
- Internationalisation or man-page generation
- Adding examples as runnable tests

### Approach

All changes are in `apm/src/main.rs` where the `Command` enum is defined.

**Mechanism**

Clap 4 (derive API) shows `long_about` when `--help` is used, and `about` (the
first doc-comment line) when `-h` is used. Add a `#[command(long_about = "...")]`
attribute to each enum variant to provide the extended text. The existing
`///` one-liner doc comment becomes the `about` and is left untouched.

For the top-level `Cli` struct, add `long_about` to the `#[command(...)]`
attribute to describe the overall tool and workflow states.

**Hide `_hook`**

Add `#[command(hide = true)]` to the `Hook` variant so it does not appear in
`apm -h` or `apm --help`.

**Content to include per command** (implementer fills in the prose):

| Command | Key points for `long_about` |
|---------|----------------------------|
| top-level | workflow overview (states), actors (agent/supervisor), two main entry points (`apm next` for agents, `apm list` for humans) |
| `init` | creates `.apm/apm.toml` and `apm.agents.md`, installs git hooks; `--migrate` for repos with root-level config |
| `list` | read-only query; filters combinable; `--all` needed for closed tickets |
| `show` | reads from the ticket branch blob; `--no-aggressive` skips fetch (faster for scripts) |
| `new` | creates branch + ticket file; agents must pass `--no-edit`; `--side-note` for out-of-scope observations during implementation |
| `state` | transitions follow the state machine in `apm.toml`; illegal transitions are rejected; use `apm show` to check current state first |
| `set` | valid fields: `priority`, `effort`, `risk`, `title`, `agent`, `supervisor`, `branch`; use `-` to clear agent/supervisor/branch |
| `start` | claims ticket (sets agent, state → `in_progress`), provisions permanent worktree, prints worktree path; `--spawn` launches a Claude subprocess |
| `next` | selects by priority desc, then id asc, among tickets in states actionable by the caller; `--json` for agent scripts |
| `sync` | fetches remote, detects merged branches → prompts to accept/close, updates local cache |
| `take` | reassigns agent field to caller; use when previous agent is gone and ticket is stuck `in_progress` |
| `worktrees` | permanent worktrees survive `apm sync`; `--add` is idempotent; always use these, never manual `git worktree add` |
| `review` | opens `$EDITOR` on the ticket file then optionally transitions state; `--to` skips the interactive prompt (useful in scripts) |
| `verify` | checks cache consistency, dangling branches; `--fix` auto-repairs what it can |
| `validate` | checks `apm.toml` correctness and cross-ticket integrity; `--fix` repairs branch-field mismatches; `--json` for CI |
| `agents` | prints `apm.agents.md`; useful for onboarding a new agent subprocess |
| `work` | loops `apm start --next --spawn`; stops when `apm next` returns null; `--dry-run` prints which tickets would be started |
| `close` | force-closes from any state; supervisor only; logs reason in history |
| `clean` | removes worktrees and local branches for closed tickets; always run `--dry-run` first |
| `spec` | `--section` alone reads; `--section --set` writes; `--check` validates required sections exist; `--mark` checks off an item |

**Order of changes**
1. Add `long_about` to `Cli` (top-level)
2. Add `#[command(hide = true)]` to `Hook`
3. Add `long_about` to each remaining variant in `Command`, following table above
4. Run `cargo test --workspace` — no tests should change (help text is not tested today)

### Open questions



### Amendment requests



### Code review
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T04:47Z | — | new | apm |
| 2026-03-30T05:19Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T05:22Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T05:40Z | specd | ready | apm |
| 2026-03-30T05:52Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T05:57Z | in_progress | implemented | claude-0330-0245-main |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |