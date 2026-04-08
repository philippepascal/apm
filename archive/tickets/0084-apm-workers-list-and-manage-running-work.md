+++
id = "0084"
title = "apm workers: list and manage running worker processes"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/0084-apm-workers-list-and-manage-running-work"
created_at = "2026-03-30T05:14:13.045392Z"
updated_at = "2026-03-30T18:07:48.168149Z"
+++

## Spec

### Problem

`apm start --spawn` launches worker processes but provides no way to observe or
manage them afterward. The only way to check on a worker is to run `ps aux |
grep claude`, inspect the ticket state with `apm show`, and guess at the log
location. There is no way to tail a worker's output, kill a stuck worker, or
see how long it has been running.

Specifically missing today:
- No registry of which workers are running and which tickets they own
- No way to tail a worker's log from the CLI
- No way to kill a worker cleanly without digging up the PID manually
- Log file exists (`.apm-worker.log` in the worktree) but its path is not
  surfaced anywhere after `apm start --spawn` prints it

### Acceptance criteria

- [x] `apm start --spawn` writes a `.apm-worker.pid` file to the worktree
  containing the worker PID and ticket ID, deleted automatically when the
  process exits
- [x] `apm workers` lists all currently running workers in a table:
  ticket ID, title, PID, elapsed time, current ticket state
- [x] `apm workers` shows no output (or "No workers running.") when no
  `.apm-worker.pid` files exist in any active worktree
- [x] `apm workers --log <id>` tails the last N lines of the worker log for
  ticket `<id>` and follows new output (like `tail -f`)
- [x] `apm workers --kill <id>` sends SIGTERM to the worker for ticket `<id>`
  and removes the `.apm-worker.pid` file; prints a confirmation
- [x] `apm workers --kill <id>` exits non-zero with a clear message if the
  worker is not running
- [x] Stale `.apm-worker.pid` files (PID no longer alive) are detected and
  reported as "crashed" in `apm workers` output rather than silently skipped
  or treated as running
- [x] `cargo test --workspace` passes

### Out of scope

- Re-attaching to a worker's interactive session
- Worker resource limits (CPU, memory)
- Remote workers (all workers are local processes)

### Approach

**PID file**

When `apm start --spawn` forks the worker, write a JSON file to the worktree:

```json
{ "pid": 85291, "ticket_id": 35, "started_at": "2026-03-30T05:14Z" }
```

at `<worktree>/.apm-worker.pid`. Use a wrapper script or a `std::process::Command`
post-spawn hook to delete the file on exit. Since `claude --print` is a
foreground process, the simplest approach: write the PID file before exec,
then in a cleanup handler (or a small wrapper) delete it when the process
exits. A shell wrapper works well:

```bash
claude --print ... ; rm -f .apm-worker.pid
```

**`apm workers` — list**

Scan all registered worktrees (`git worktree list --porcelain`) for
`.apm-worker.pid` files. For each:

1. Parse PID and ticket ID from JSON
2. Check if PID is alive (`kill -0 <pid>`)
   - Alive → show as running with elapsed time
   - Dead → show as "crashed" (stale pid file)
3. Load ticket state via `apm show <id>` for the current state column

Output format:

```
ID    TITLE                        PID    STATE        ELAPSED
35    github-apm-meta              85291  in_progress  42m
77    help text audit              —      crashed      —
```

**`apm workers --log <id>`**

Locate the worktree for ticket `<id>`, then exec `tail -f
<worktree>/.apm-worker.log`. Exit with an error if the log file doesn't exist.

**`apm workers --kill <id>`**

Locate the `.apm-worker.pid` for ticket `<id>`, send SIGTERM to the PID,
remove the file, print `"killed worker for ticket #<id> (PID <pid>)"`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T05:14Z | — | new | claude-0330-0245-main |
| 2026-03-30T05:14Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T05:15Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T05:18Z | specd | ready | apm |
| 2026-03-30T05:53Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T06:12Z | in_progress | implemented | claude-0329-1200-b4e7 |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |