+++
id = "498febe0"
title = "Worker leaks edits into main worktree; capture full transcript"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/498febe0-worker-leaks-edits-into-main-worktree-ca"
created_at = "2026-04-28T22:35:14.876837Z"
updated_at = "2026-04-28T22:42:08.457067Z"
+++

## Spec

### Problem

**Incident:** ticket 63f5e6d2 ("UI: epics filter fixes") landed in `merge_failed` because the main worktree at `/Users/philippepascal/repos/apm/` had uncommitted changes to `apm-ui/src/components/supervisor/SupervisorView.tsx` — exactly the file the worker had also been editing in its ticket worktree. `git status` confirmed the leak. The ticket branch had its own clean commit (`ed85ddec`); the main worktree had an unstaged duplicate that git refused to overwrite during the implicit auto-merge.

The CLAUDE.md rule "main worktree always on main, never holds work" was violated. The worker pipeline is somehow allowing edits to land outside the ticket worktree.

`.apm-worker.log` only captured the agent's final stdout summary (13 lines) — no tool-call detail, no path information, no way to forensically reconstruct which Edit/Write call hit which path. We can't tell whether (a) the worker spawned with the wrong cwd, (b) the worker's Edit calls resolved absolute paths against the main worktree by mistake, or (c) something else. Without the transcript we have to guess.

**What this ticket should do:**

1. **Capture the full agent transcript in `.apm-worker.log`.** Today the spawn captures only stdout. Include stderr and the jsonl tool-call stream from Claude Code (or whatever the configured worker driver is). The log should let a future investigator see every Edit/Write/Bash call the worker issued, with the paths it touched. Make this the default; no flag needed.

2. **Investigate and harden the worker spawn cwd.** Confirm in `apm-core/src/start.rs` (the spawn code path used by `apm start --spawn`) that the spawned process's CWD is set to the ticket worktree — not the repo root, not the parent shell's cwd. If it's already set correctly, the bug is elsewhere; the investigation should still produce a regression test that exercises a fresh `apm start --spawn`, captures the spawned process's actual cwd, and asserts it matches the worktree.

3. **Update `apm.worker.md` to make worktree-path discipline explicit.** The instruction file should call out: never edit files outside the ticket's own worktree path; always use absolute paths rooted at the worktree (not at the main repo root). Show concrete examples of the right vs wrong path forms. **This .md change must land in both `apm-core/src/default/apm.worker.md` (the template future `apm init` writes) and the project's current `.apm/apm.worker.md` (what running workers actually read). The two must stay in sync.** The same rule applies to any other instruction file modified by future tickets.

4. **Workflow.toml sync — add `merge_failed` state to the project config.** `apm-core/src/default/workflow.toml:201-212` already defines the state with transitions to `implemented` (retry merge) and `in_progress` (go back). The project's `.apm/workflow.toml` does NOT have this state, so a ticket that lands in `merge_failed` cannot transition out via `apm state` — the state is unreachable in the configured workflow. Port the state block from the default template into the project's current config. **Like the .md fix, this must land in both `apm-core/src/default/workflow.toml` (already correct, no change) and `.apm/workflow.toml` (currently missing the state).**

**Out of scope:**

- A defensive check in `apm state … implemented` that fails fast when the main worktree is dirty for files the ticket changed. (User explicitly excluded this.)
- An automatic mechanism to keep `apm-core/src/default/*.md` and `apm-core/src/default/workflow.toml` in sync with existing project `.apm/` files (the upgrade-path problem more broadly). Out of scope here; could be a follow-up.
- Recovering ticket 63f5e6d2 itself. (Operational, not a code change.)

**Acceptance pointers (for the spec phase):**

- After this ticket lands, running a worker that mistakenly issues an Edit against the main repo root produces a log entry showing the absolute path used; investigators can identify the leak.
- A spawn-cwd regression test exists and passes.
- `apm.worker.md` in both locations contains the path-discipline guidance with examples.
- `.apm/workflow.toml` contains the `merge_failed` state with both transitions; `apm state <merge_failed_ticket> implemented` succeeds (retries merge); `apm state <merge_failed_ticket> in_progress` succeeds (back to in-progress).

### Acceptance criteria

- [ ] Running a worker with the default configuration produces a `.apm-worker.log` containing JSONL lines (one JSON object per line) for every event emitted by the worker driver, including `tool_use` events that expose the path argument for every Edit, Write, and Bash call the worker made
- [ ] A `.apm-worker.log` from a run that edits a file contains a line from which the absolute path of the edited file can be extracted (e.g. via grep or jq)
- [ ] An integration test in `apm-core/tests/` (or `apm/tests/integration.rs`) spawns a worker process via the same code path used by `apm start --spawn`, captures the spawned process's working directory, and asserts it equals the ticket's worktree path
- [ ] `cargo test --workspace` passes with the new integration test included
- [ ] `.apm/apm.worker.md` contains a Path discipline section that states: never edit files outside the ticket worktree; always use absolute paths rooted at the worktree path shown in `apm show`; includes a labelled correct example and a wrong example
- [ ] `apm-core/src/default/apm.worker.md` contains the identical Path discipline section (same wording as the project file)
- [ ] `.apm/workflow.toml` contains a `[[workflow.states]]` block with `id = "merge_failed"`, `actionable = ["supervisor"]`, and two `[[workflow.states.transitions]]` entries: one to `implemented` and one to `in_progress`, both `trigger = "manual"`
- [ ] `apm state <ticket_in_merge_failed> implemented` exits 0
- [ ] `apm state <ticket_in_merge_failed> in_progress` exits 0

### Out of scope

- A defensive guard in `apm state … implemented` that fails fast when the main worktree is dirty for files the ticket touched (explicitly excluded by supervisor)
- An automatic mechanism to keep `apm-core/src/default/*.md` and `apm-core/src/default/workflow.toml` in sync with existing project `.apm/` files on upgrades (broader upgrade-path problem; could be a follow-up ticket)
- Recovering or retrying ticket 63f5e6d2 itself (operational, not a code change)
- Identifying the exact root cause of the original main-worktree leak in ticket 63f5e6d2 beyond what the new logging will reveal in future incidents

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T22:35Z | — | new | philippepascal |
| 2026-04-28T22:36Z | new | groomed | philippepascal |
| 2026-04-28T22:42Z | groomed | in_design | philippepascal |