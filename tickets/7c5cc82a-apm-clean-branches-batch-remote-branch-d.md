+++
id = "7c5cc82a"
title = "apm clean --branches: batch remote branch deletions into a single push"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5cc82a-apm-clean-branches-batch-remote-branch-d"
created_at = "2026-06-03T03:03:55.652009Z"
updated_at = "2026-06-03T06:32:24.138193Z"
+++

## Spec

### Problem

PROBLEM: apm clean --branches deletes remote ticket branches by calling git push origin --delete branch one branch at a time, serially. For projects with many ticket branches to clean (the supervisor observed this in syn), each push incurs:

- A full SSH or HTTPS connection setup
- A pre-push hook invocation locally
- A pre-receive and post-receive hook invocation on the server (GitHub or self-hosted)
- A network round-trip to confirm the delete

With N branches, total cost is N times each. In syn (where some epics had hundreds of ticket branches accumulated over time), this made apm clean take minutes per session. Git natively supports deleting multiple refs in one push via multiple refspecs, which collapses the cost to a single connection + a single hook cycle + a single round-trip regardless of N.

CODE CHAIN (current):

- apm/src/cmd/clean.rs::run with --branches=true iterates clean candidates and calls clean::remove(candidate, ...) per candidate
- apm-core/src/clean.rs::remove at lines 346-353: when remote_branch_exists is true on the candidate, calls git_util::delete_remote_branch
- apm-core/src/git_util.rs::delete_remote_branch at lines 1040-1043: runs git push origin --delete branch (one push per branch)

PROPOSED CHANGE (option A from the conversation):

Collect every branch that is eligible for remote deletion across all candidates, then issue a SINGLE batched git push origin --delete refs/heads/A refs/heads/B refs/heads/C ... at the end of the cleanup pass. The local cleanup steps (worktree removal, local branch delete, prune_remote_tracking) stay in the per-candidate loop because they are fast and have independent failure modes.

NEW HELPER: git_util::delete_remote_branches(branches: &[&str]) -> Result<DeleteBranchesOutput>

- Take a slice of branch names
- Construct a single git push origin --delete refs/heads/branch1 refs/heads/branch2 ... invocation
- Return per-branch success or failure (git push emits per-ref status lines)
- Empty input: return Ok with empty output, no push attempted

REFACTORED FLOW in clean.rs::run or clean::remove:

- Iterate candidates as today; for each, perform LOCAL cleanup (worktree remove, local branch delete, prune remote tracking)
- Collect branches into a Vec<String> when candidate.remote_branch_exists is true
- After the per-candidate loop, call delete_remote_branches on the collected list
- Apply prune_remote_tracking for each branch that succeeded (currently done inside delete_remote_branch's caller; the spec-writer should preserve this semantic)
- Surface per-branch failures via the existing warnings channel

KEEP the existing single-branch delete_remote_branch function for callers that still want per-branch semantics (apm epic close uses it, for example). Don't ripple this change into them; epic close usually deletes one branch.

ACCEPTANCE CRITERIA hints (for spec-writer to refine):

- A new public function git_util::delete_remote_branches(branches: &[&str]) exists; empty input returns Ok with empty result, no git invocation
- delete_remote_branches with N branches issues exactly one git push command (testable by spawning a wrapper script or counting via a mock)
- apm clean --branches with N remote-eligible candidates issues exactly one git push for remote deletion regardless of N
- A failure deleting one ref does not block local cleanup of other candidates; per-branch failures appear in the warnings channel as before
- apm epic close still uses single-branch delete_remote_branch with no behaviour change
- Integration test: create a temp repo with origin remote and several remote-only ticket branches; run apm clean --branches; assert all remote branches are gone and the test asserts the underlying push count is 1 (via a spy/intercept or by counting git reflog entries)
- All existing cargo test --workspace tests pass

OUT OF SCOPE:

- A --no-remote flag for skipping the remote delete entirely (separate concern; could be filed later if needed)
- Parallel pushes (the batch makes this moot)
- Changing apm epic close's single-branch delete behaviour
- Changing the local cleanup path
- Changing what counts as remote_branch_exists (the existing ls-remote check at candidate-collection time is preserved)
- Detecting and warning when a branch is in a protected-branch ruleset on origin (out of band concern)
- Changing apm sync's behaviour around remote branches

REFERENCES:

- apm/src/cmd/clean.rs (the --branches entry point)
- apm-core/src/clean.rs::remove around lines 306-358 (the per-candidate remove)
- apm-core/src/git_util.rs::delete_remote_branch around lines 1040-1043 (the single-branch helper to keep)
- Background: supervisor observed multi-minute apm clean times in syn project due to per-branch serial pushes; identified that git supports multi-ref delete in one push

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
| 2026-06-03T03:03Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
