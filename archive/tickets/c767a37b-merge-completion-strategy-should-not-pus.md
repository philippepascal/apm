+++
id = "c767a37b"
title = "Merge completion strategy should not push main to origin"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "1915"
branch = "ticket/c767a37b-merge-completion-strategy-should-not-pus"
created_at = "2026-04-02T03:15:29.694878Z"
updated_at = "2026-04-02T19:07:47.341978Z"
+++

## Spec

### Problem

When a state transition with `completion = "merge"` is executed (e.g. `apm state <id> implemented`), the merge completion strategy performs five steps:\n\n1. Push the ticket branch to origin\n2. Fetch the default branch from origin\n3. Find the correct merge directory\n4. Merge the ticket branch into the default branch locally\n5. **Push the default branch to origin**\n\nStep 5 is the problem. Pushing `main` (or the configured default branch) to origin is a supervisor action — it publishes the merged work publicly and is a destructive, non-reversible operation. An autonomous agent completing a ticket should not have this authority. The push should be left to the human supervisor, who can review the local merge state before deciding to publish.

### Acceptance criteria

- [x] After `apm state <id> implemented` with `completion = "merge"`, the ticket branch is pushed to origin\n- [x] After `apm state <id> implemented` with `completion = "merge"`, the ticket branch is merged into the local default branch\n- [x] After `apm state <id> implemented` with `completion = "merge"`, the default branch is NOT pushed to origin\n- [x] The local default branch (e.g. `main`) reflects the merge after the transition completes\n- [x] All existing tests continue to pass

### Out of scope

- The `pr` completion strategy (unchanged — PR creation/merge via GitHub API is a separate flow)\n- The `pull` and `none` completion strategies (unaffected)\n- Any UI or output changes to indicate that a manual push is now required\n- Adding a separate command or flag to let supervisors trigger the push later

### Approach

Single change in one file:\n\n**`apm-core/src/state.rs`** — `merge_into_default` function (approx. lines 207–253)\n\nRemove the `git push origin <default_branch>` block (lines 242–249). The function already returns `Ok(())` after the merge; simply delete the push command and its error-handling branch.\n\nBefore:\n```rust\nlet push = std::process::Command::new("git")\n    .args(["push", "origin", default_branch])\n    .current_dir(&merge_dir)\n    .output()?;\n\nif !push.status.success() {\n    bail!("push failed: {}", String::from_utf8_lossy(&push.stderr).trim());\n}\n```\n\nAfter: *(lines deleted — nothing replaces them)*\n\nNo other files need to change. The integration test in `apm/tests/e2e.rs` may assert that main is pushed to origin after `apm state implemented`; if so, update the assertion to verify the push did NOT happen (or remove the push-verification step and replace it with a check that the local default branch contains the merge commit).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T03:15Z | — | new | apm |
| 2026-04-02T16:55Z | new | groomed | apm |
| 2026-04-02T16:56Z | groomed | in_design | philippepascal |
| 2026-04-02T16:57Z | in_design | specd | claude-0402-1700-sp3c |
| 2026-04-02T17:22Z | specd | ready | apm |
| 2026-04-02T17:23Z | ready | in_progress | philippepascal |
| 2026-04-02T17:24Z | in_progress | implemented | claude-0402-1730-w0rk |
| 2026-04-02T19:07Z | implemented | closed | apm-sync |