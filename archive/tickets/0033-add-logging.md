+++
id = 33
title = "add-logging"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "apm"
agent = "claude-0327-2000-33dd"
branch = "ticket/0033-add-logging"
created_at = "2026-03-27T21:08:32.355155Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

APM performs git commits, branch operations, worktree management, and state transitions, but produces no persistent audit trail. In multi-agent scenarios — where several processes may be acting concurrently on different tickets — it is difficult to reconstruct after the fact what each agent did and when. A simple append-only log file would provide that visibility without requiring changes to the existing data model or git history.

### Acceptance criteria

- [ ] `apm.toml` supports a `[logging]` section with `enabled = true/false` and `file = "<path>"` (path relative to repo root or absolute)
- [ ] When `enabled = false` or the section is absent, there is zero overhead — no file is opened, no allocations per action
- [ ] Each log entry is one line: `<RFC-3339 timestamp> [<agent>] <action> <detail>`
- [ ] Git operations logged: `commit_to_branch`, `add_worktree`, `remove_worktree`, `fetch_all`, `push_ticket_branches`, `next_ticket_id`
- [ ] State transitions logged (from `apm state` command)
- [ ] The log file is opened in append mode; existing content is never truncated
- [ ] `apm init` includes a commented-out `[logging]` example in the generated `apm.toml`

### Out of scope

- Log rotation or size limits
- Log levels (debug/info/error)
- Structured or JSON log format
- Remote log sinks
- Log parsing or querying commands (`apm log`)
- File locking across concurrent processes

### Approach

Add a `LoggingConfig` struct in `apm-core/src/config.rs` with `enabled: bool` and `file: Option<PathBuf>`. Add a `logger` module in `apm-core/src/` with an `ApmLogger` struct wrapping a `BufWriter<File>`. Initialize once in `apm/src/main.rs` after config load, stored in a `static OnceLock<Mutex<ApmLogger>>`. Provide a `log_action(agent, action, detail)` free function that no-ops when disabled. Call it at key git operation sites in `git.rs` and at state transition sites in the CLI commands.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:08Z | — | new | apm |
| 2026-03-28T01:02Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T01:05Z | specd | ready | apm |
| 2026-03-28T02:09Z | ready | in_progress | claude-0327-2000-33dd |
| 2026-03-28T02:11Z | in_progress | implemented | claude-0327-2000-33dd |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |