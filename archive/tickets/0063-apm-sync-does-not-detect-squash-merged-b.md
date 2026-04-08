+++
id = 63
title = "apm sync does not detect squash-merged branches"
state = "closed"
priority = 1
effort = 2
risk = 1
author = "philippepascal"
agent = "claude-0329-1430-main"
branch = "ticket/0063-apm-sync-does-not-detect-squash-merged-b"
created_at = "2026-03-29T22:50:59.530523Z"
updated_at = "2026-03-30T05:24:13.847237Z"
+++

## Spec

### Problem

`apm sync` detects merged branches via `git branch --merged`, which only identifies branches whose tip commit is an ancestor of the default branch. Squash merges produce a single new commit in main; the original branch commits are not ancestors, so `merged_into_main()` in `git.rs` misses them. Squash-merged tickets are never transitioned to `accepted` and accumulate indefinitely in the branch list.

GitHub's default merge strategy for most repos is squash merge, making this a common failure case.

### Acceptance criteria

- [x] `apm sync` detects branches that have been squash-merged into the default branch and treats them identically to regular merges (offers to transition the ticket to `accepted`)
- [x] Regular (non-squash) merge detection is unchanged
- [x] Branches with no commits yet merged are not falsely detected
- [x] `cargo test --workspace` passes

### Out of scope

- Rebase-merge detection (similar problem, separate ticket if needed)
- Changing the interactive accept prompt or auto-accept behaviour

### Approach

In `apm-core/src/git.rs`, extend `merged_into_main` with a second pass for branches not caught by `--merged`.

For each ticket branch not already in the `--merged` results, run:

```
git log --cherry-pick --right-only --no-merges <default_branch>...<branch>
```

Git's `--cherry-pick` flag suppresses commits whose patch-id already appears on the other side of the `...`. `--right-only` shows only commits from `<branch>`. If the output is empty, every commit in the branch has an equivalent patch already in `<default_branch>` — the branch was squash-merged.

Implementation sketch:

```rust
pub fn merged_into_main(root: &Path, default_branch: &str) -> Result<Vec<String>> {
    // ... existing --merged logic ...
    let already_merged: HashSet<String> = /* existing result */;

    // Second pass: squash-merge detection for remaining ticket branches.
    let mut squash_merged = Vec::new();
    for branch in ticket_branches(root)? {
        if already_merged.contains(&branch) { continue; }
        let range = format!("{default_branch}...{branch}");
        let out = run(root, &[
            "log", "--cherry-pick", "--right-only", "--no-merges",
            "--format=%H", &range,
        ]).unwrap_or_default();
        if out.trim().is_empty() {
            squash_merged.push(branch);
        }
    }

    Ok(already_merged.into_iter().chain(squash_merged).collect())
}
```

Use the remote ref (`origin/<default_branch>`) if available, same as the existing logic.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T22:50Z | — | new | philippepascal |
| 2026-03-30T01:08Z | new | in_progress | claude-0329-1430-main |
| 2026-03-30T01:29Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T04:38Z | implemented | accepted | apm |
| 2026-03-30T05:24Z | accepted | closed | apm-sync |