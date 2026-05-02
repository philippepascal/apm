+++
id = "29cc63a1"
title = "Pre-merge leak detection: refuse apm state implemented when main has uncommitted overlap"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29cc63a1-pre-merge-leak-detection-refuse-apm-stat"
created_at = "2026-05-01T02:30:13.061854Z"
updated_at = "2026-05-02T03:14:20.748628Z"
+++

## Spec

### Problem

When a worker writes to the main worktree (intentional leak or bug), the bad change sits there until someone notices via `git status` or fails an `apm state implemented` merge. The deferred enforcement piece from ticket 498febe0's spec is what closes this gap.

**Incident pattern:**
1. Worker spawns into its ticket worktree.
2. Worker (despite path-discipline guidance in apm.worker.md) issues a tool call with an absolute path pointing at the main worktree.
3. The call may succeed (if the file is in the project's allowlist or the worker was spawned with -P) or fail (default permission denial). When it succeeds, the change is silent.
4. Later, when the supervisor runs `apm state X implemented`, the merge of the ticket branch into main aborts because the main worktree has uncommitted changes that would be overwritten — but the error message is git's stock "Aborting" which doesn't point at the worker that caused it.
5. Cleanup requires the supervisor to identify the leaked file, decide whether to commit/discard, and re-attempt the merge.

**This ticket adds a pre-merge check that catches the leak earlier with a clearer diagnostic.**

**Reference:** ticket 498febe0's spec (already implemented) explicitly listed this as out of scope ("a defensive check in apm state implemented that fails fast when the main worktree is dirty for files the ticket changed"). Now is the time.

**Should land after the wrapper epic (4312fbd4)** so the wrapper-side path validator (separate ticket) and this check are layered together.

**Scope:**
- In `apm-core/src/state.rs`, before the merge attempt in the `Merge` and `PrOrEpicMerge` completion strategies:
  - Compute the set of files modified on the ticket branch since its merge-base with the target (main, or the epic branch).
  - Run `git status --porcelain` on the target worktree.
  - If any of the modified files appear in the status output as uncommitted: refuse the transition with a clear diagnostic naming each leaked file, the ticket id, and a pointer to the worker's transcript at `<worktree>/.apm-worker.log`.
  - On clean: proceed with the merge as today.
- The check is informational — does not modify the working tree or revert changes.
- New error message format:
  ```
  cannot complete <transition>: main worktree has uncommitted changes to files this ticket also modified:
    apm-ui/src/components/foo.tsx
    .apm/config.toml
  This usually means a worker leaked edits outside its worktree.
  Inspect the worker's transcript: <ticket-worktree>/.apm-worker.log
  Then either commit/restore the leaked files in main and re-run apm state <id> implemented, or run apm verify to investigate.
  ```

**Out of scope:**
- Auto-recovering the leak (move uncommitted changes to a stash, etc.). The supervisor decides; this ticket only surfaces.
- Pre-spawn checks (the leak hasn't happened yet).
- Wrapper-layer interception of tool calls (separate ticket).

**Acceptance pointers:**
- Integration test: simulate a leak by creating an uncommitted edit in the main worktree on a file the ticket branch also modified. `apm state X implemented` exits non-zero with the new diagnostic. The exit text names the leaked file. The ticket state remains at `in_progress` (no transition occurred).
- Integration test: clean main worktree → `apm state X implemented` proceeds normally.
- Integration test: the `Pr` and `None` completion strategies (no merge attempted) are unaffected.

### Acceptance criteria

- [ ] `apm state X implemented` exits non-zero and prints a diagnostic when the merge-target worktree has uncommitted changes to at least one file that also appears in `git diff --name-only <merge-base>..<ticket-branch>`
- [ ] The diagnostic names every overlapping file (one per line, indented)
- [ ] The diagnostic includes the ticket id
- [ ] The diagnostic includes the path `<ticket-worktree>/.apm-worker.log` (or a generic placeholder when the worktree is not found)
- [ ] When the check fires, the ticket state on the branch remains unchanged (no `implemented` commit, no `on_failure` rollback commit)
- [ ] `apm state X implemented` succeeds normally when the merge-target worktree has no uncommitted changes
- [ ] `apm state X implemented` succeeds normally when the merge-target worktree has uncommitted changes to files that are NOT on the ticket branch (no false positives)
- [ ] The check runs for the `Merge` completion strategy (direct merge to `target_branch` or `default_branch`)
- [ ] The check runs for the `PrOrEpicMerge` completion strategy when `target_branch` is set (epic-branch merge path)
- [ ] `apm state X implemented` is unaffected when the completion strategy is `Pr`, `Pull`, or `None` (no merge is attempted; no check runs)
- [ ] When `check_leaked_files` cannot resolve the merge-base (e.g. no shared history), it returns an empty list and the transition is not blocked
- [ ] When the merge-target worktree does not exist on disk yet, the check returns empty and does not block the transition

### Out of scope

- Auto-recovery of leaked edits (stashing, reverting, or moving changes on behalf of the user)
- Pre-spawn checks to prevent workers from leaking in the first place
- Wrapper-layer tool-call interception (separate ticket in epic 4312fbd4)
- Checking for leaks in worktrees other than the merge target (e.g. other ticket worktrees)
- The `Pull` completion strategy (pulls upstream into ticket branch; no merge of ticket into target)
- The `Pr` strategy's no-merge path (no worktree merge is attempted)
- Detecting leaks that have already been committed to the main branch (those cause a normal merge conflict, not a worker leak)
- `apm verify` command (referenced in the diagnostic as a follow-up tool; implementation is out of scope here)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:14Z | groomed | in_design | philippepascal |