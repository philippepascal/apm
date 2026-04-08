+++
id = "f5bee9f9"
title = "refactor: move cleanup logic from clean.rs into apm-core"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "claude-0330-0245-main"
agent = "47523"
branch = "ticket/f5bee9f9-refactor-move-cleanup-logic-from-clean-r"
created_at = "2026-03-30T14:27:36.851282Z"
updated_at = "2026-03-30T18:09:08.816339Z"
+++

## Spec

### Problem

clean.rs (171 lines) contains all cleanup detection and orchestration logic as a single CLI command. This logic — terminal state resolution, merged branch detection via git branch --merged, ancestor checking via git merge-base --is-ancestor, ticket state cross-checking between the ticket branch and main, remote tip agreement checking, worktree dirty-checking, and local branch existence checking — belongs in apm-core.

These are pure data checks on git state, not CLI presentation concerns. Embedding them in the CLI command prevents apm-serve from reusing them: the server will need to show a 'ready to clean' list and trigger cleanup without shelling out to the apm binary.

The target is apm_core::clean::candidates() returning a structured list of branches safe to remove (with reasons), and apm_core::clean::remove() performing the actual deletion. The CLI becomes thin: call candidates(), format output, prompt, then call remove().

### Acceptance criteria

- [x] `apm_core::clean::candidates()` exists and is `pub` in apm-core
- [x] `apm_core::clean::remove()` exists and is `pub` in apm-core
- [x] `candidates()` returns a `CleanCandidate` for each ticket branch that is in a terminal state and merged into the default branch
- [x] `candidates()` skips branches where the worktree has uncommitted changes
- [x] `candidates()` skips branches where local and remote tips disagree
- [x] `candidates()` skips branches where ticket state on the ticket branch differs from state on main
- [x] `candidates()` returns an empty list when there is nothing to clean
- [x] `remove()` deletes the worktree (if present) and the local branch for a given candidate
- [x] `apm clean --dry-run` prints the same output as before, now driven by `candidates()`
- [x] `apm clean` (non-dry-run) removes the same set of worktrees and branches as before, now via `remove()`
- [x] All six existing clean integration tests pass without modification

### Out of scope

- `apm-serve` integration (a future ticket will consume the new API)
- New CLI flags, prompting changes, or output format changes
- Changes to the state machine or how terminal states are configured
- Squash-merge detection changes (existing `merged_into_main()` logic is unchanged)
- Any new tests beyond ensuring existing ones still pass

### Approach

**1. Create `apm-core/src/clean.rs`**

Define a `CleanCandidate` struct capturing everything needed to describe and act on a cleanup:

```rust
pub struct CleanCandidate {
    pub ticket_id: String,
    pub ticket_title: String,
    pub branch: String,
    pub worktree: Option<PathBuf>,   // Some if a worktree exists locally
    pub reason: String,              // Human-readable: "closed, merged"
}
```

Implement `pub fn candidates(root: &Path, config: &Config) -> anyhow::Result<Vec<CleanCandidate>>`:
- Determine terminal states from `config.workflow.states` (those with `terminal: true`) plus hardcoded `"closed"`
- Load all tickets with `ticket::load_all_from_git(root)`
- Get merged branches with `git::merged_into_main(root, &config.project.default_branch)`
- For each ticket whose state is terminal:
  - Skip if branch is not in merged set
  - Skip if branch tip is not an ancestor of default branch (`git::is_ancestor`)
  - Skip if ticket state on default branch differs from state on ticket branch (`ticket::state_from_branch`)
  - Warn (log) but skip if local and remote tips disagree (`git::branch_tip` vs `git::remote_branch_tip`)
  - Skip if worktree has uncommitted changes (check via `git status --porcelain` in the worktree)
  - Skip if neither a worktree nor a local branch exists
  - Otherwise, append a `CleanCandidate`

Implement `pub fn remove(root: &Path, candidate: &CleanCandidate) -> anyhow::Result<()>`:
- If `candidate.worktree` is `Some`, call `git::remove_worktree(root, path)`
- Delete the local branch with `git branch -d`

**2. Export from `apm-core/src/lib.rs`**

Add `pub mod clean;` to `lib.rs`.

**3. Rewrite `apm/src/cmd/clean.rs`**

Replace the 171-line body with:
- Call `apm_core::clean::candidates(root, config)?`
- If `dry_run`: iterate and print each candidate's branch and reason; return
- Otherwise: for each candidate, print what is being removed, call `apm_core::clean::remove(root, candidate)?`
- Print "Nothing to clean." if the list is empty

All dirty-check, ancestor-check, state-check logic moves out; `clean.rs` keeps only formatting and the dry-run flag.

**4. Keep integration tests green**

The existing six tests in `apm/tests/integration.rs` test through the CLI (`apm clean`), so they exercise the new core functions transitively. No test changes expected.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
| 2026-03-30T16:35Z | in_design | specd | claude-0330-1631-6640 |
| 2026-03-30T16:58Z | specd | ready | philippepascal |
| 2026-03-30T17:25Z | ready | in_progress | philippepascal |
| 2026-03-30T17:29Z | in_progress | implemented | claude-0330-1725-87d0 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:09Z | accepted | closed | apm-sync |