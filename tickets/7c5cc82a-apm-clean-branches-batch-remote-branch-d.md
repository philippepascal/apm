+++
id = "7c5cc82a"
title = "apm clean --branches: batch remote branch deletions into a single push"
state = "implemented"
priority = 0
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5cc82a-apm-clean-branches-batch-remote-branch-d"
created_at = "2026-06-03T03:03:55.652009Z"
updated_at = "2026-06-03T21:03:53.858482Z"
+++

## Spec

### Problem

`apm clean --branches` deletes remote ticket branches by calling `git push origin --delete <branch>` once per candidate, serially. Each push incurs a full connection setup (SSH or HTTPS handshake), a local pre-push hook invocation, a remote pre-receive/post-receive cycle, and a network round-trip for the acknowledgement. With N branches, total wall-clock cost is N × that overhead. In projects where hundreds of ticket branches accumulate across epics, this makes `apm clean` take minutes per session.

Git natively supports deleting multiple refs in a single push: `git push origin --delete refs/heads/A refs/heads/B refs/heads/C ...` collapses the cost to a single connection, single hook cycle, and single round-trip regardless of N. The fix is to collect all remote-eligible branches across the candidate loop and issue one batched push after the loop, rather than one push inside the loop.

### Acceptance criteria

- [x] `git_util::delete_remote_branches` called with an empty slice returns `Ok` immediately without spawning a git process
- [x] `git_util::delete_remote_branches` with N > 0 branches issues exactly one `git push` command containing all N refspecs
- [x] `apm clean --branches` with N remote-eligible candidates issues exactly one `git push` for remote deletion, regardless of N
- [x] A failure deleting one remote ref in the batch does not prevent the remaining refs from being deleted; each per-ref failure appears as a warning on stderr
- [x] `prune_remote_tracking` is called for each successfully deleted remote branch after the batch push
- [x] `apm clean --branches --dry-run` prints "would remove branch" lines for remote-eligible candidates and issues no `git push`
- [x] `apm-server` clean handler behaviour is unchanged: it continues calling `clean::remove` with per-branch remote deletion (no batching)
- [x] All existing `cargo test --workspace` tests pass

### Out of scope

- `--no-remote` flag for skipping remote deletion entirely (separate concern)
- Parallel pushes (the batch makes this moot)
- Batching remote deletions in `apm-server`'s maintenance handler
- Changing `apm epic close`'s remote branch deletion (it inlines its own `git push --delete` and is unchanged)
- Changing the local cleanup path inside `clean::remove` (worktree removal, local branch delete, local prune)
- Changing what counts as `remote_branch_exists` (the `ls-remote` check at candidate-collection time is preserved)
- Detecting or warning about protected branches on origin
- Changing `apm sync` behaviour around remote branches

### Approach

#### New helper: `git_util::delete_remote_branches`

Add to `apm-core/src/git_util.rs`:

```rust
pub struct DeleteBranchesOutput {
    pub deleted: Vec<String>,           // branch names successfully deleted
    pub failed: Vec<(String, String)>,  // (branch, reason) for each failure
}
```

`delete_remote_branches(root: &Path, branches: &[&str]) -> Result<DeleteBranchesOutput>`:

- Empty input: return `Ok(DeleteBranchesOutput { deleted: vec![], failed: vec![] })` with no git invocation.
- Build and run: `git push --porcelain origin --delete refs/heads/b1 refs/heads/b2 ...` using `Command` directly (not `run()`, which bails on non-zero exit; a partial failure makes the overall exit code non-zero).
- Parse `--porcelain` stdout line-by-line:
  - Lines starting with `-\t` → success; extract branch name by taking the second tab-field and stripping the leading `:` and `refs/heads/` prefix.
  - Lines starting with `!\t` → failure; extract branch name the same way; take the third tab-field (after the second `\t`) as the reason.
- If stdout yielded no parsed results and the exit code is non-zero, treat all input branches as failed with the full stderr as the reason (handles total network failure or missing remote).
- Return `Ok(DeleteBranchesOutput { deleted, failed })` — `Err` only when git cannot be spawned.

#### `clean::remove` signature change

Add a `skip_remote_delete: bool` parameter to `clean::remove` in `apm-core/src/clean.rs`:

```rust
pub fn remove(root: &Path, candidate: &CleanCandidate, force: bool,
              remove_branches: bool, skip_remote_delete: bool) -> Result<RemoveOutput>
```

Guard the existing `if candidate.remote_branch_exists { ... }` block with `&& !skip_remote_delete`. All logic inside that block (the `delete_remote_branch` call, the `prune_remote_tracking` on success, and the warning on error) is unchanged — it simply becomes unreachable when the CLI passes `skip_remote_delete: true`.

Update the server caller at `apm-server/src/handlers/maintenance.rs:207` to pass `false` (no behaviour change).

#### Batch remote deletion in `apm/src/cmd/clean.rs::run`

Before the candidate loop, declare:
```rust
let mut remote_to_delete: Vec<String> = Vec::new();
```

In the loop, for each code path that calls `clean::remove` (both the force-confirmed path and the normal path):
- Pass `skip_remote_delete: branches` (i.e. `true` when `--branches` is set).
- After the `clean::remove` call, if `branches && candidate.remote_branch_exists`, push `candidate.branch.clone()` onto `remote_to_delete`.

In the dry-run path: no change — no git push is ever issued.

After the loop, if `!dry_run && !remote_to_delete.is_empty()`:
```rust
let refs: Vec<&str> = remote_to_delete.iter().map(|s| s.as_str()).collect();
match git_util::delete_remote_branches(root, &refs) {
    Ok(out) => {
        for branch in &out.deleted {
            git_util::prune_remote_tracking(root, branch);
        }
        for (branch, reason) in &out.failed {
            eprintln!("warning: could not delete remote branch {branch}: {reason}");
        }
    }
    Err(e) => eprintln!("warning: batch remote branch deletion failed: {e}"),
}
```

#### Tests

Unit test in `apm-core/src/git_util.rs`: call `delete_remote_branches` with `&[]` in a temp dir — verify it returns `Ok` without invoking git (no panic, no filesystem side effect).

Integration test in `apm/tests/integration.rs`:
1. Create a temp bare repo as the origin and a working clone.
2. Push two or more `ticket/*` branches to origin.
3. Create closed-state tickets on the default branch so `clean::candidates` picks them up.
4. Call `apm clean --branches` against the working clone.
5. Assert `git ls-remote origin ticket/*` returns empty — all remote ticket branches gone.
6. Assert no remote branches remain that were in the candidate set.

The single-push property is verified by reading the `delete_remote_branches` implementation: it constructs exactly one `Command::new("git").args(["push", "--porcelain", "origin", "--delete", ...])` call. If stricter test coverage is needed, the test can write a counter script to `GIT_EXEC_PATH` before invoking the clean operation.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-03T03:03Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:46Z | groomed | in_design | philippepascal |
| 2026-06-03T06:52Z | in_design | specd | claude |
| 2026-06-03T20:47Z | specd | ready | philippepascal |
| 2026-06-03T20:57Z | ready | in_progress | philippepascal |
| 2026-06-03T21:03Z | in_progress | implemented | claude |
