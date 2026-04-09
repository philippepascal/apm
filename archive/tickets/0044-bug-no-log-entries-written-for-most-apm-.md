+++
id = 44
title = "Bug: no log entries written for most apm commands"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0328-1000-a1b2"
agent = "claude-0328-t44a"
branch = "ticket/0044-bug-no-log-entries-written-for-most-apm-"
created_at = "2026-03-28T08:58:40.578900Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The logger is correctly initialized in `main.rs` when `logging.enabled = true`
(the log file is created and `init()` is called). However, `logger::log()` is
only called inside `apm-core/src/git.rs` — specifically inside the git commit
helpers (`commit_to_branch`, `commit_files_to_branch`). These helpers are
invoked when tickets transition state and commits are written to branches.

Commands that do not write commits produce zero log output:
- `apm list`, `apm show`, `apm next` — read-only, never call git helpers
- `apm sync --offline` — reads branches, no commits
- `apm verify` — read-only

Even write commands like `apm state` (when committing to a branch) call the git
helper, so they _do_ log — but only at the git layer, not at the command layer.
There is no record of which command was invoked, by whom, with what arguments.

The result: the log file is created but effectively empty during normal use.

### Acceptance criteria

- [x] Every `apm` command invocation writes at least one log entry when logging
  is enabled — at minimum: `cmd <subcommand> <args>` at entry
- [x] State transitions log the event: `state <id> <from> -> <to>`
- [x] Log entries use the existing format:
  `<timestamp> [<agent>] <action> <detail>`
- [x] Read-only commands (`list`, `show`, `next`, `verify`, `agents`) log their
  invocation
- [x] No new log calls are needed inside `git.rs` (existing ones stay)

### Out of scope

- Structured / JSON log format
- Log levels (debug, info, warn, error)
- Logging failures or error results (only successful invocations need logging)
- Performance impact (logging is append-only and already behind a mutex)

### Approach

**`apm/src/main.rs`**: after `logger::init`, add a single log call before
dispatching to the subcommand. Build the detail string from the raw CLI args:

```rust
// After init:
let args: Vec<String> = std::env::args().skip(1).collect();
apm_core::logger::log("cmd", &args.join(" "));
```

This produces one log line per invocation for every command, covering the
read-only commands that currently produce no output.

**`apm-core/src/ticket.rs` or `apm/src/cmd/state.rs`**: add a log call after a
successful state transition. The state command already calls
`git::commit_to_branch` which logs at the git level; add a higher-level entry:

```rust
apm_core::logger::log("state", &format!("{id} {} -> {}", old_state, new_state));
```

This is the only command-level log call that adds value beyond the invocation
log. Other commands (set, take, start, sync) are sufficiently covered by the
command-entry log plus the existing git-layer logs.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:58Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T09:03Z | new | specd | apm |
| 2026-03-28T19:19Z | specd | ready | apm |
| 2026-03-28T19:25Z | ready | in_progress | claude-0328-t44a |
| 2026-03-28T19:26Z | in_progress | implemented | claude-0328-t44a |
| 2026-03-28T19:29Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |