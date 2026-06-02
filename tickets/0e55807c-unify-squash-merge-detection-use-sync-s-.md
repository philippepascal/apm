+++
id = "0e55807c"
title = "Unify squash-merge detection: use sync's helper in apm epic close and apm clean --epics"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0e55807c-unify-squash-merge-detection-use-sync-s-"
created_at = "2026-06-02T01:12:18.100370Z"
updated_at = "2026-06-02T06:06:48.048169Z"
+++

## Spec

### Problem

PROBLEM: apm has three places that ask 'is this branch merged into main', and they use different algorithms with different correctness properties.

1. apm-core/src/git_util.rs::merged_into_main + squash_merged (used by apm sync): handles both regular merges and squash merges by synthesizing a virtual squash commit (git commit-tree branch tree dash-p merge_base) and checking via git cherry. Prefers origin slash default-branch when available.

2. apm/src/cmd/epic.rs::run_close lines 161 to 172 (the already-merged shortcut): naive check via git log --oneline default-branch dot dot branch — non-empty output means not merged. Misses squash merges entirely.

3. apm/src/cmd/epic.rs::run_epic_clean lines 509 to 516 (the is_merged closure): same naive git log check as run_close. Same blind spot.

The inconsistency surfaces in this scenario, observed by the supervisor in the syn project:
- Empty epic 72294403 has a PR open
- First apm epic close 722 succeeds (creates the PR)
- Supervisor merges the PR on GitHub (squash merge — the GitHub default)
- Second apm epic close 722 fails: gh pr create rejects with GraphQL No commits between main and epic
- apm clean --epics says Nothing to clean
- Result: the branch is orphaned — GitHub treats it as merged; apm cannot detect or clean it. The supervisor has to hand-delete with git branch dash-D plus git push origin --delete.

DESIGN: extract sync's content-merge detection into a shared helper and replace both naive checks.

1. In apm-core/src/git_util.rs, expose pub fn is_branch_content_merged(root, default_branch, branch) -> Result<bool>. This should mirror what squash_merged already does for a single branch:
   - merge_base = git merge-base default_branch branch
   - branch_tip = git rev-parse branch carat braces commit
   - If branch_tip equals merge_base: return true (already an ancestor, caught by --merged path)
   - Synthesize virtual squash commit: git commit-tree branch carat braces tree dash-p merge_base dash-m squash
   - git cherry default_branch virtual_squash — return true if output starts with dash
   - Prefer origin slash default_branch over local when the remote ref exists. This mirrors merged_into_main's preference order (apm-core/src/git_util.rs around line 102).

2. In apm/src/cmd/epic.rs::run_close, replace the inline git log check (around lines 161 to 172) with a call to apm_core::git_util::is_branch_content_merged.

3. In apm/src/cmd/epic.rs::run_epic_clean, replace the is_merged closure (around lines 509 to 516) with a call to the same helper.

4. The existing squash_merged in git_util.rs takes a Vec<String> of candidates. Refactor it to call is_branch_content_merged in a loop. Keep the public API (function signature) of merged_into_main unchanged so apm sync is not disrupted.

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- A new public function is_branch_content_merged in apm-core/src/git_util.rs that returns true for both regular merges and squash merges
- The helper prefers origin slash default-branch when available, falls back to local default-branch
- apm epic close on a squash-merged empty epic skips PR creation and deletes the branch (instead of attempting to push and failing)
- apm clean --epics offers a squash-merged empty epic as a candidate for deletion
- Existing apm sync behaviour is unchanged; merged_into_main and squash_merged still work as today
- A unit test covers: regular-merged branch returns true; squash-merged branch returns true; truly unmerged branch returns false
- An integration test exercises apm epic close on a squash-merged scenario and verifies branch deletion without PR
- An integration test exercises apm clean --epics on a squash-merged empty epic and verifies it appears as a candidate

OUT OF SCOPE:
- Changing apm sync's behaviour or its existing helpers' shape beyond the refactor needed to share the new helper
- Detecting non-squash variants of GitHub merges (rebase merge, merge queue) beyond what squash_merged already handles
- UI changes (this is a CLI fix only)
- Handling the case where origin slash default-branch is unreachable or stale (the existing fallback to local default-branch is preserved)
- Changes to how apm epic close decides whether the branch is mergeable at all (separate concern)

REFERENCES:
- apm-core/src/git_util.rs::merged_into_main (around line 102) and squash_merged (around line 217)
- apm/src/cmd/epic.rs::run_close (lines 161 to 172, the already-merged shortcut)
- apm/src/cmd/epic.rs::run_epic_clean (lines 509 to 516, the is_merged closure)
- Background: the supervisor observed this with epic 72294403 in syn project, where a squash-merged branch ended up orphaned in apm's view
WORKTREE CLEANUP REQUIREMENT: when apm clean --epics (or apm epic close after a successful merge detection) determines a branch should be deleted, it must also remove any worktree pinning that branch. Git refuses branch deletion while a worktree exists for it (error: cannot delete branch X used by worktree at Y). Today the supervisor has to run three separate commands by hand (git worktree remove, git branch -D, git push origin --delete). The new code path must do all three. The worktree-removal logic already exists in apm-core/src/worktree.rs (apm uses it for ticket worktree cleanup); the epic cleanup path should reuse it. The spec-writer should add an AC explicitly covering this end-to-end cleanup.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T01:12Z | — | new | philippepascal |
| 2026-06-02T06:06Z | new | closed | philippepascal |
