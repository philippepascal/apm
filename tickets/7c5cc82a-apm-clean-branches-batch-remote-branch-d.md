+++
id = "7c5cc82a"
title = "apm clean --branches: batch remote branch deletions into a single push"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5cc82a-apm-clean-branches-batch-remote-branch-d"
created_at = "2026-06-03T03:03:55.652009Z"
updated_at = "2026-06-03T06:46:04.593572Z"
+++

## Spec

### Problem

`apm clean --branches` deletes remote ticket branches by calling `git push origin --delete <branch>` once per candidate, serially. Each push incurs a full connection setup (SSH or HTTPS handshake), a local pre-push hook invocation, a remote pre-receive/post-receive cycle, and a network round-trip for the acknowledgement. With N branches, total wall-clock cost is N × that overhead. In projects where hundreds of ticket branches accumulate across epics, this makes `apm clean` take minutes per session.

Git natively supports deleting multiple refs in a single push: `git push origin --delete refs/heads/A refs/heads/B refs/heads/C ...` collapses the cost to a single connection, single hook cycle, and single round-trip regardless of N. The fix is to collect all remote-eligible branches across the candidate loop and issue one batched push after the loop, rather than one push inside the loop.

### Acceptance criteria

- [ ] `git_util::delete_remote_branches` called with an empty slice returns `Ok` immediately without spawning a git process
- [ ] `git_util::delete_remote_branches` with N > 0 branches issues exactly one `git push` command containing all N refspecs
- [ ] `apm clean --branches` with N remote-eligible candidates issues exactly one `git push` for remote deletion, regardless of N
- [ ] A failure deleting one remote ref in the batch does not prevent the remaining refs from being deleted; each per-ref failure appears as a warning on stderr
- [ ] `prune_remote_tracking` is called for each successfully deleted remote branch after the batch push
- [ ] `apm clean --branches --dry-run` prints "would remove branch" lines for remote-eligible candidates and issues no `git push`
- [ ] `apm-server` clean handler behaviour is unchanged: it continues calling `clean::remove` with per-branch remote deletion (no batching)
- [ ] All existing `cargo test --workspace` tests pass

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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-03T03:03Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
| 2026-06-03T06:46Z | groomed | in_design | philippepascal |