+++
id = "81f4fa74"
title = "propose better handling of merge failures by worker"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/81f4fa74-propose-better-handling-of-merge-failure"
created_at = "2026-04-18T07:37:39.058963Z"
updated_at = "2026-04-18T19:01:56.164442Z"
+++

## Spec

### Problem

When the worker transitions `in_progress → implemented`, the completion strategy runs `git merge --no-ff` into the default branch. If the merge fails (conflict, push error, etc.), the entire state transition fails with an error and the ticket remains in `in_progress`. The supervisor has no way to distinguish "worker is still implementing" from "worker finished but the merge blew up." The failure reason is only visible in the stderr of whoever ran `apm state`, which in an agent-driven workflow is ephemeral.

The desired behaviour is that merge failure is a first-class outcome: the ticket moves to a dedicated state, the failure reason is persisted in the ticket file, and the supervisor can act on it directly from `apm review` or `apm list` without needing to re-run commands or inspect git logs.

### Acceptance criteria

- [x] When `git merge` fails during the `in_progress → implemented` transition, the ticket transitions to `merge_failed` instead of staying in `in_progress`
- [x] The merge error message (stderr from git) is written to a `### Merge notes` section in the ticket file before the state is committed
- [x] `merge_failed` is supervisor-actionable: it appears in `apm review` output and `apm list` under the supervisor role
- [x] `apm show <id>` renders the `### Merge notes` section when the ticket is in `merge_failed` state
- [x] From `merge_failed`, the supervisor can transition to `implemented` without triggering another merge attempt
- [ ] From `merge_failed`, the supervisor can transition back to `in_progress` (to let the worker retry)
- [ ] When the transition to `merge_failed` itself fails (e.g. cannot commit the ticket), the original merge error is still reported and the ticket is left in `in_progress` (no silent data loss)

### Out of scope

- Automatic merge conflict resolution or retry logic
- Failures during `git push` (before the merge step) — only the merge step itself is covered
- PR-based completion strategies (`completion = "pr"`, `completion = "pr_or_epic_merge"`)
- Notifying the supervisor via push notification or any async channel — the state change in the ticket is the only signal
- Reusing the existing `blocked` state for merge failures (a dedicated state is used instead)
- UI changes in `apm-ui` beyond what `apm show` already renders from ticket sections

### Approach

**1. Add `merge_failed` state to `apm-core/src/default/workflow.toml`**

Add a new state entry (after `implemented`):

```toml
[states.merge_failed]
actionable = ["supervisor"]
label = "Merge failed"

[[states.merge_failed.transitions]]
to = "implemented"
completion = "none"    # supervisor resolved the conflict manually; no auto-merge

[[states.merge_failed.transitions]]
to = "in_progress"
completion = "none"    # worker will retry
```

`merge_failed` does **not** set `worker_end` or `satisfies_deps` — those only apply once the merge actually lands.

**2. Catch merge errors in `apm-core/src/state.rs` `transition()`**

In the `CompletionStrategy::Merge` arm (lines ~144–178), wrap the `merge_into_default()` call so that on error it:

- Writes the failure reason (git stderr) to the ticket's `### Merge notes` section, creating it if absent, using the existing ticket-file mutation helpers.
- Commits the updated ticket file to the current branch (same commit pattern used elsewhere in `transition()`).
- Moves the ticket to `merge_failed` by invoking the state-write path with `completion = "none"` so no further merge is attempted.
- Returns `Ok(())` so the outer caller sees a clean exit.

If the fallback commit or state write themselves fail, return the original merge error so the ticket stays in `in_progress` (no silent data loss).

**3. Render `### Merge notes` in `apm show`**

In `apm/src/cmd/show.rs`, when the ticket is in `merge_failed` state, extract `### Merge notes` from the ticket body and display it prominently (error-style header or red colour if the terminal supports it). If `show` already renders all sections generically, this may be a no-op — confirm before adding code.

**4. Verify supervisor workflows pick up the new state**

`apm review` and `apm list` filter on `actionable = ["supervisor"]`. Confirm they pick up `merge_failed` automatically from the workflow config. No code change expected, but include a test or manual check.

**Order of changes:**
1. `workflow.toml` — add the state (immediately testable with `apm state` manually)
2. `state.rs` — catch merge errors, write `### Merge notes`, transition to `merge_failed`
3. `show.rs` — render merge notes prominently (may be a no-op)
4. Integration test — trigger a merge conflict and assert ticket ends in `merge_failed` with error in `### Merge notes`

**Constraints:**
- `### Merge notes` must be written in the same commit that records `state = "merge_failed"` — do not split.
- The existing `git merge --abort` cleanup in `merge_into_default()` (`git_util.rs` line ~976) stays in place; worktree is always left clean.

### 1. Add `merge_failed` state to `apm-core/src/default/workflow.toml`

Add a new state entry after `implemented`:

```toml
[states.merge_failed]
actionable = ["supervisor"]
label = "Merge failed"

[[states.merge_failed.transitions]]
to = "implemented"
completion = "none"    # supervisor resolved the conflict manually; no auto-merge

[[states.merge_failed.transitions]]
to = "in_progress"
completion = "none"    # worker will retry
```

`merge_failed` does **not** set `worker_end` or `satisfies_deps` — those only apply once the merge actually lands.

### 2. Catch merge errors in `apm-core/src/state.rs` `transition()`

In the `CompletionStrategy::Merge` arm (lines ~144–178), wrap the `merge_into_default()` call so that on error it:

1. Writes the failure reason to the ticket's `### Merge notes` section (create the section if absent, overwrite if present) using the existing ticket-file mutation helpers.
2. Commits the updated ticket file to the current branch (same commit pattern used elsewhere in `transition()`).
3. Calls `transition()` recursively (or directly invokes the state-write path) to move the ticket to `merge_failed`, with `completion = "none"` so no further merge is attempted.
4. Returns `Ok(())` so the outer caller sees a clean exit — the state is now `merge_failed` in the ticket file.

If step 2 or 3 themselves fail, fall back to returning the original merge error so the ticket stays in `in_progress` and nothing is silently lost.

### 3. Render `### Merge notes` in `apm show`

In `apm/src/cmd/show.rs`, after rendering the existing ticket sections, check if the ticket is in `merge_failed` state. If so, extract `### Merge notes` from the ticket body and display it prominently (e.g. with an error-style header or red colour if the terminal supports it). No change needed if the section is already rendered generically — confirm by reading `show.rs` first.

### 4. Ensure `merge_failed` surfaces in supervisor workflows

Verify `apm review` and `apm list` pick up `actionable = ["supervisor"]` automatically (they should, since they already filter on that field). No code change expected beyond the workflow.toml entry, but add a test or manual check.

### Order of changes

1. `workflow.toml` — add the state (no code changes needed yet, makes it testable via `apm state` manually)
2. `state.rs` — catch merge errors and transition to `merge_failed`
3. `show.rs` — render merge notes (may be a no-op if show already renders all sections)
4. Integration test — add a test that triggers a merge conflict and asserts the ticket ends in `merge_failed` with the error in `### Merge notes`

### Constraints

- The `### Merge notes` section must be written before the state changes so that the commit that records `state = "merge_failed"` also contains the notes. Do not split across two commits.
- The existing `git merge --abort` cleanup in `merge_into_default()` (`git_util.rs` line ~976) stays in place — the worktree is always left clean regardless of this change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T07:37Z | — | new | philippepascal |
| 2026-04-18T18:42Z | new | groomed | philippepascal |
| 2026-04-18T18:42Z | groomed | in_design | philippepascal |
| 2026-04-18T18:47Z | in_design | specd | claude-0418-1842-f9a8 |
| 2026-04-18T18:58Z | specd | ready | philippepascal |
| 2026-04-18T19:01Z | ready | in_progress | philippepascal |