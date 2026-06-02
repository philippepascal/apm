+++
id = "dc2b08db"
title = "apm move should not change the current worktree checkout"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/dc2b08db-apm-move-should-not-change-the-current-w"
created_at = "2026-06-02T03:20:39.058642Z"
updated_at = "2026-06-02T18:39:31.600107Z"
+++

## Spec

### Problem

`apm move <ticket-id> <epic-id>` correctly reassigns the ticket to the new epic but leaves the main worktree's HEAD pointing at the ticket branch. The supervisor has to run `git checkout main` to recover their working state after every invocation.

The root cause is in `apm-core/src/ticket/ticket_util.rs::move_to_epic`, step 9. The implementation calls `git rebase --onto <newbase> <upstream> <branch>` with the three-argument form. Git's three-argument rebase checks out `<branch>` in the current worktree before replaying commits — this is what switches HEAD. Other ticket-mutating commands (`apm set`, `apm spec`, `apm state`) avoid this problem by using `commit_to_branch` / `try_worktree_commit`, which operate via temporary worktrees and never touch the calling worktree's HEAD.

The fix is to run the rebase inside a temporary worktree, exactly as `try_worktree_commit` does. After the rebase the local `refs/heads/<ticket_branch>` ref is updated to the rebased tip; the main worktree's HEAD is never touched. `commit_to_branch` (called immediately after) already operates safely without a checkout, so steps 10+ require no changes.

### Acceptance criteria

- [x] Running `apm move <id> <epic-id>` from the main worktree with HEAD on `main` leaves HEAD on `main` when the command returns
- [x] The ticket file on the ticket branch contains the updated `epic` field after the move, matching today's behaviour
- [x] A history row (`move: main → epic/…`) is appended to the ticket branch after the move, with no regression in audit trail
- [x] Uncommitted changes present in the main worktree before `apm move` are still present and uncommitted after the command returns
- [x] An integration test in `apm/tests/integration.rs` creates a ticket, creates an epic, runs `apm move` with HEAD on `main`, and asserts HEAD is still `main` after the call

### Out of scope

- Semantic changes to what `apm move` does — the ticket is still rebased onto the new epic branch and the epic field is still updated
- Behaviour when `apm move` is run from inside the ticket's own worktree (separate concern; that case is already guarded by the step-6 "branch checked out in a worktree" check)
- apm-server and apm-ui (CLI-only bug)
- Rebase conflict handling — the existing error message and abort logic is preserved unchanged

### Approach

#### Change to `move_to_epic` (step 9)

**File:** `apm-core/src/ticket/ticket_util.rs`

Replace the current three-argument rebase that runs in the main worktree:

```rust
crate::git_util::run(root, &["rebase", "--onto", &new_base_sha, &old_upstream_sha, &ticket_branch])
```

with a temporary-worktree rebase that keeps the main worktree's HEAD intact:

1. Resolve the local SHA for `ticket_branch`:
   ```rust
   let branch_sha = crate::git_util::run(root, &["rev-parse", &format!("refs/heads/{ticket_branch}")])?;
   ```

2. Build a temp path using the same naming convention as `try_worktree_commit` (pid + counter + sanitised branch name) under `std::env::temp_dir()`.

3. Create a detached temporary worktree at that SHA, then check out the branch inside it (same two-step pattern used in `try_worktree_commit` and `commit_files_to_branch`):
   ```rust
   crate::git_util::run(root, &["worktree", "add", "--detach", &wt_path_str, &branch_sha])?;
   let _ = crate::git_util::run(&wt_path, &["checkout", "-B", &ticket_branch]);
   ```

4. Run the two-argument rebase from inside the temp worktree (no branch argument → git rebases the currently checked-out branch, updating `refs/heads/<ticket_branch>` in place):
   ```rust
   let rebase_result = crate::git_util::run(&wt_path, &["rebase", "--onto", &new_base_sha, &old_upstream_sha]);
   ```

5. On failure, abort in the temp worktree, clean it up, then surface the existing error messages unchanged.

6. On success, clean up the temp worktree:
   ```rust
   let _ = crate::git_util::run(root, &["worktree", "remove", "--force", &wt_path_str]);
   let _ = std::fs::remove_dir_all(&wt_path);
   ```

Steps 1–8 and 10+ of `move_to_epic` are unchanged. The step-6 guard ("Reject if branch is checked out in a worktree") is preserved — it prevents `checkout -B` in step 3 above from failing with "already checked out."

After the rebase `refs/heads/<ticket_branch>` points to the rebased tip. `commit_to_branch` (step 10) uses `try_worktree_commit` to commit the frontmatter update, which already handles the branch safely without touching the main worktree.

#### Integration test

**File:** `apm/tests/integration.rs`

Add a test `move_does_not_change_main_worktree_head`:

1. Call `init_repo()` to get a temp repo on `main`.
2. Create an epic branch directly via git (following the pattern in `setup_with_epic()`).
3. Create a ticket with `apm new`.
4. Assert HEAD is `main`.
5. Call `run_apm(dir.path(), &["move", &ticket_id, &epic_id])`.
6. Assert `git branch --show-current` is still `"main"`.
7. Assert the ticket file on the ticket branch contains `epic = "<epic_id>"`.

No fixture files are needed. The test uses only the existing `init_repo`, `create_ticket`, and `git` helpers already in the file.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T03:20Z | — | new | philippepascal |
| 2026-06-02T06:07Z | new | groomed | philippepascal |
| 2026-06-02T06:07Z | groomed | in_design | philippepascal |
| 2026-06-02T06:11Z | in_design | specd | claude |
| 2026-06-02T17:41Z | specd | ready | philippepascal |
| 2026-06-02T17:41Z | ready | in_progress | philippepascal |
| 2026-06-02T17:46Z | in_progress | implemented | claude |
| 2026-06-02T18:39Z | implemented | closed | philippepascal(apm-sync) |
