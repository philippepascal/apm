+++
id = "498febe0"
title = "Worker leaks edits into main worktree; capture full transcript"
state = "implemented"
priority = 0
effort = 6
risk = 5
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/498febe0-worker-leaks-edits-into-main-worktree-ca"
created_at = "2026-04-28T22:35:14.876837Z"
updated_at = "2026-04-29T01:28:37.796521Z"
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

- [x] Running a worker with the default configuration produces a `.apm-worker.log` containing JSONL lines (one JSON object per line) for every event emitted by the worker driver, including `tool_use` events that expose the path argument for every Edit, Write, and Bash call the worker made
- [x] A `.apm-worker.log` from a run that edits a file contains a line from which the absolute path of the edited file can be extracted (e.g. via grep or jq)
- [x] `apm start --spawn` exits non-zero with a descriptive error message if the configured worker binary does not advertise `--output-format stream-json` in its `--help` output; the error names the missing flag and the binary path
- [x] An integration test in `apm-core/tests/` (or `apm/tests/integration.rs`) spawns a worker process via the same code path used by `apm start --spawn`, captures the spawned process's working directory, and asserts it equals the ticket's worktree path
- [x] `cargo test --workspace` passes with the new integration test included
- [x] `.apm/apm.worker.md` contains a Path discipline section that states: never edit files outside the ticket worktree; always use absolute paths rooted at the worktree path shown in `apm show`; includes a labelled correct example and a wrong example
- [x] `apm-core/src/default/apm.worker.md` contains the identical Path discipline section (same wording as the project file)
- [x] A `cargo test` assertion in `apm-core/tests/` reads both `apm-core/src/default/apm.worker.md` and `.apm/apm.worker.md` and fails with a diff if their contents are not byte-for-byte identical

### Out of scope

- A defensive guard in `apm state … implemented` that fails fast when the main worktree is dirty for files the ticket touched (explicitly excluded by supervisor)
- An automatic mechanism to keep `apm-core/src/default/*.md` and `apm-core/src/default/workflow.toml` in sync with existing project `.apm/` files on upgrades (broader upgrade-path problem; could be a follow-up ticket)
- Recovering or retrying ticket 63f5e6d2 itself (operational, not a code change)
- Identifying the exact root cause of the original main-worktree leak in ticket 63f5e6d2 beyond what the new logging will reveal in future incidents
- Porting the `merge_failed` state to `.apm/workflow.toml` — delegated to ticket `79a03767` (`apm validate --fix` will detect and port missing referenced states from the default template)

### Approach

**1. Full transcript capture in `.apm-worker.log`**

File: `apm-core/src/start.rs` — `build_spawn_command` (non-container) and the container spawn function.

Current state: Both stdout and stderr are already redirected to `.apm-worker.log` via file descriptor cloning. The gap is that `claude -p` without further flags emits only a final text summary to stdout; tool-call events (Edit paths, Write paths, Bash commands) are not emitted.

Fix: Add `--output-format stream-json` to the args vector when building the worker command. This causes the Claude CLI to emit a newline-delimited JSON stream of all session events — including `tool_use` events with their full argument payloads and `tool_result` events — to stdout. Since stdout is already redirected to `.apm-worker.log`, no other plumbing change is needed.

Worker-driver compatibility check (addresses Amendment 2): Before inserting `--output-format stream-json` into the args, probe the configured worker binary by running `<binary> --help` and scanning its combined stdout+stderr for the string `--output-format`. If the string is absent, `build_spawn_command` (or its caller) must return an error — do not fall back silently. The error message must include: the binary path, the missing flag name, and a hint to upgrade the binary or configure an alternative worker command. This makes the hard dependency explicit at spawn time rather than producing a silently incomplete log.

Locate the point in `build_spawn_command` where the args vec is assembled. First add the probe, then (if it passes) insert `--output-format`, `stream-json`. Apply the identical probe + insertion to the container spawn function.

---

**2. Spawn-cwd regression test**

Finding: `start.rs` line 170 already calls `cmd.current_dir(wt)` — the CWD is already correct. No production-code change is needed. The goal is a regression test.

In `apm-core/tests/` or `apm/tests/integration.rs`, add a test that:
- Configures a minimal worker command (e.g. `sh -c 'pwd > /tmp/apm-cwd-check.txt'`) in place of the real worker
- Calls the same spawn code path used by `apm start --spawn`
- Waits for the child to exit
- Reads the output file and asserts the trimmed contents equal the expected worktree path

Check existing test helpers before writing setup from scratch; reuse any worktree or ticket fixtures already present. If `build_spawn_command` is private, add a `cfg(test)` re-export or test via the public start function with a mock worker command.

---

**3. `apm.worker.md` path-discipline section**

Add a new `Path discipline` section to BOTH files:
- `apm-core/src/default/apm.worker.md`
- `.apm/apm.worker.md`

Place it after the existing shell-discipline section. The two files must be word-for-word identical for this section.

Content: Your working directory is the ticket worktree. Never read or write files outside it. Always use absolute paths rooted at your worktree. The worktree path appears in `apm show <id>` under Worktree — note it at the start of your run. Include a labelled correct example (path inside the worktree) and a wrong example (path in the main repo root). End with: if a tool call resolves to a path outside your worktree, stop immediately, file a side-note ticket, and set yourself to blocked.

---

**3a. `apm.worker.md` sync enforcement test (addresses Amendment 1)**

After editing both `apm.worker.md` files, add a Rust test in `apm-core/tests/` that:
- Reads the byte content of `apm-core/src/default/apm.worker.md` (relative to the crate root via `env!("CARGO_MANIFEST_DIR")`)
- Reads the byte content of `.apm/apm.worker.md` (relative to the workspace root, one level up from the crate)
- Asserts they are byte-for-byte identical; on failure, prints a unified diff so the developer sees what diverged

This test enforces the sync rule going forward: any future ticket that edits one file but not the other will fail `cargo test --workspace`.

---

**Order of steps:**
1. Add Path discipline section to both `apm.worker.md` files; confirm they match word-for-word
2. Add the `apm.worker.md` sync test to `apm-core/tests/`
3. Verify `--output-format stream-json` flag is supported (`<worker-binary> --help`)
4. Add compatibility probe + `--output-format stream-json` args in both spawn functions in `start.rs`
5. Write spawn-cwd regression test in `apm-core/tests/`
6. Run `cargo test --workspace` — all tests must pass
7. Commit and transition to implemented

Note: `498febe0` and `e1781eef` (UI: show tickets in merge_failed state) no longer block each other — neither is a prerequisite for the other. Both depend on `79a03767`'s `apm validate --fix` having been run against the project to port the `merge_failed` state into `.apm/workflow.toml`; that is an operational step, not a code-level dependency between these tickets.

### Open questions


### Amendment requests

- [x] AC must include enforcement of the .md sync rule (Problem says "future .md changes must land in both default template and project config" but no AC enforces it going forward). Pick one of: (a) add a section to `CONTRIBUTING.md` documenting the sync rule, or (b) add a CI test that diffs `apm-core/src/default/*.md` against any project `.apm/*.md` of the same name and fails on drift. Either is acceptable; (b) is stronger but requires a fixture project for the diff target.
- [x] AC must include a worker-driver compatibility check. Spec proposes `claude --output-format stream-json` for transcript capture but does not make it a precondition. If the installed `claude` binary on the developer/CI machine does not support that flag, the worker spawn breaks at runtime. Either (a) add an AC asserting `claude --help` contains `--output-format stream-json` and the implementation refuses to spawn workers without it, or (b) implement detect-and-fall-back: try the flag, on failure capture only stdout (current behavior) and log a warning. (a) is more honest about the dependency; (b) preserves graceful degradation. Pick one.
- [x] Document the implementation ordering relative to ticket `e1781eef` (UI does not show tickets in merge_failed state). Both tickets touch the merge_failed plumbing — `498febe0` ports the state into the project's `.apm/workflow.toml`, `e1781eef` makes the UI surface it. Recommend that `498febe0` lands first so the UI ticket is not surfacing a state that is not yet in the local config. Add a one-line note to the Approach (no schema change needed) acknowledging the ordering preference.
- [x] Drop the workflow.toml port scope from this ticket. The original Problem item #4 ("port `merge_failed` state to `.apm/workflow.toml`") is now subsumed by ticket `79a03767`'s extended `apm validate --fix`, which will detect missing referenced states and port them from the default template. Remove the corresponding step from the Approach and the AC item that asserts `merge_failed` is present in the project's workflow.toml. Update Out of Scope to note that workflow.toml content management is delegated to `79a03767`.
- [x] Update the cross-ticket note about `e1781eef` ordering: previously this ticket was the prerequisite for `e1781eef` (because it ported the state). Now neither blocks the other; both depend on `79a03767`'s `--fix` having been run on the project, which is an operational step rather than a code dependency. Re-assess the ordering recommendation.
- [x] The .md sync rule (apm.worker.md must stay identical between default template and project config) STAYS in this ticket — that is an instruction-file content concern, separate from `79a03767`'s state-machine concern. No change to AC #8 or the related Approach section.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T22:35Z | — | new | philippepascal |
| 2026-04-28T22:36Z | new | groomed | philippepascal |
| 2026-04-28T22:42Z | groomed | in_design | philippepascal |
| 2026-04-28T22:48Z | in_design | specd | claude-0428-2242-af90 |
| 2026-04-28T23:00Z | specd | ammend | philippepascal |
| 2026-04-28T23:01Z | ammend | in_design | philippepascal |
| 2026-04-28T23:04Z | in_design | specd | claude-0428-2301-6e28 |
| 2026-04-28T23:14Z | specd | ammend | philippepascal |
| 2026-04-28T23:23Z | ammend | in_design | philippepascal |
| 2026-04-28T23:27Z | in_design | specd | claude-0428-2323-70b0 |
| 2026-04-28T23:31Z | specd | ready | philippepascal |
| 2026-04-28T23:56Z | ready | in_progress | philippepascal |
| 2026-04-29T01:23Z | in_progress | ready | philippepascal |
| 2026-04-29T01:23Z | ready | in_progress | philippepascal |
| 2026-04-29T01:28Z | in_progress | implemented | claude-0429-0123-8368 |
