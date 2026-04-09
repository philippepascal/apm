+++
id = "268a88c9"
title = "apm state accepted: pull default branch after PR merge"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "71249"
branch = "ticket/268a88c9-apm-state-accepted-pull-default-branch-a"
created_at = "2026-04-01T06:10:14.360337Z"
updated_at = "2026-04-01T07:12:30.401515Z"
+++

## Spec

### Problem

When a ticket transitions from `implemented` to `accepted`, GitHub has already merged the closing PR (enforced by the `pr_all_closing_merged` precondition). At that point the local `main` branch is stale — it does not yet reflect the squash-merge that GitHub performed. Nothing in APM currently fetches or fast-forwards local `main` after acceptance.

The existing `completion = "merge"` strategy is not appropriate here: it merges the ticket branch locally with `--no-ff`, creating a redundant local merge commit on top of a squash-merge that GitHub already did. What is needed instead is a lightweight pull: fetch `origin/<default_branch>` and fast-forward the local default branch to match it.

This matters because engineers who immediately branch off `main` after accepting a ticket will be working from a stale base, risking conflicts and confusion about what has actually shipped.

### Acceptance criteria

- [x] `apm state <id> accepted` fetches `origin/main` (or the configured default branch) from the remote\n- [x] After `apm state <id> accepted`, the local default branch is fast-forwarded to match `origin/<default_branch>`\n- [x] If the local default branch cannot be fast-forwarded (e.g. it has diverged), `apm state` prints a clear warning but does not fail — the state transition still completes\n- [x] `completion = "pull"` is accepted in `.apm/config.toml` transition definitions without a parse error\n- [x] `apm verify` lists the `implemented → accepted` transition as `completion: implemented → accepted = pull`\n- [x] The `implemented → accepted` transition in the project `.apm/config.toml` uses `completion = "pull"`

### Out of scope

- Removing or modifying the existing `completion = "merge"` strategy
- Deleting the ticket branch after acceptance (that belongs to the `closed` transition)
- Pushing the ticket branch to origin as part of acceptance (the PR is already merged)
- Handling cases where `git fetch` fails due to network issues (a warning and non-fatal continuation is sufficient)
- Any changes to the `implemented → closed` or `accepted → closed` transitions

### Approach

**1. Add `Pull` variant — `apm-core/src/config.rs` (line ~33)**

Add `Pull` to the `CompletionStrategy` enum alongside `Pr`, `Merge`, `None`. Serde's `rename_all = "lowercase"` attribute already maps variant names to lowercase TOML strings, so no parser changes are needed beyond adding the variant.

**2. Implement `pull_default` helper — `apm-core/src/state.rs`**

Add a private `pull_default(root: &Path, default_branch: &str) -> Result<()>`:
1. Run `git fetch origin <default_branch>`. On network failure print a warning and `return Ok(())` (non-fatal).
2. Determine the working directory: if `HEAD` is already `default_branch`, use `root`; otherwise call `git::find_worktree_for_branch(root, default_branch)` and fall back to `root` if not found (same pattern as `merge_into_default`).
3. Run `git merge --ff-only origin/<default_branch>` in that directory.
4. If the ff-only fails (local branch has diverged), print `"warning: could not fast-forward <default_branch> — pull manually"` and return `Ok(())` (non-fatal).

**3. Wire into the completion match — `apm-core/src/state.rs` (line ~135)**

Add a `CompletionStrategy::Pull` arm to the existing `match completion` block. No `push_branch` call is needed — the ticket branch was already pushed as part of the PR flow before the PR was merged.

**4. Update `verify.rs` — `apm/src/cmd/verify.rs` (line ~27)**

Add `CompletionStrategy::Pull => "pull"` to the match in the completion-strategy report loop so `apm verify` output includes the new strategy.

**5. Update `.apm/config.toml` — `implemented → accepted` transition (line ~264)**

Add `completion = "pull"` to the transition block:

```toml
[[workflow.states.transitions]]
to            = "accepted"
trigger       = "manual"
actor         = "engineer"
preconditions = ["pr_all_closing_merged"]
completion    = "pull"
```

**6. Add a unit test — `apm-core/src/config.rs`**

Extend the existing `CompletionStrategy` serde test to assert that `"pull"` deserialises to `CompletionStrategy::Pull`.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T06:10Z | — | new | philippepascal |
| 2026-04-01T06:12Z | new | in_design | philippepascal |
| 2026-04-01T06:17Z | in_design | specd | claude-0401-0612-b280 |
| 2026-04-01T06:25Z | specd | ready | apm |
| 2026-04-01T06:39Z | ready | in_progress | philippepascal |
| 2026-04-01T06:43Z | in_progress | implemented | claude-0401-0639-19d0 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |