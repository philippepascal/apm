+++
id = "fd4552ea"
title = "Refactor epic lifecycle: submit (PR/merge) vs close (cleanup); sync surfaces hints"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fd4552ea-refactor-epic-lifecycle-submit-pr-merge-"
created_at = "2026-06-02T06:05:04.230173Z"
updated_at = "2026-06-02T06:11:46.888297Z"
+++

## Spec

### Problem

GOAL: split today's apm epic close into two distinct, single-purpose commands and add passive detection to apm sync so the supervisor's mental model matches the ticket lifecycle.

CURRENT PROBLEM: apm epic close does two completely different things depending on the epic's state:
- If not merged into main: pushes the branch and opens a PR (or merges, with --merge / --auto flags)
- If already merged: deletes the local branch and skips the PR

The supervisor has to run the same command twice (once to open the PR, once to clean up after the merge). This is inconsistent with how tickets work — for tickets, the supervisor runs apm state implemented to open a PR, then apm sync (passive) detects the merge and offers to close the ticket. For epics, there is no passive detection; the supervisor has to remember to re-run apm epic close.

Two observations sharpened the design:
- In the syn project, an empty epic (72294403) was squash-merged on GitHub. apm epic close on a second invocation failed with No commits between main and epic (gh rejected the PR creation); apm clean --epics said Nothing to clean. The supervisor had to hand-delete the branch and worktree. The naive git log --oneline main..branch check that both apm clean and apm epic close used misses squash merges; sync already has the right detection but its scope did not cover epic branches.
- Conceptually, the two phases of today's apm epic close are different actions with different verbs. Submitting an epic for merge is a creation action (pushes commits, opens a PR). Closing the epic is a cleanup action (deletes the branch, removes the worktree). Calling them both close conflates them.

NEW MODEL:

apm epic submit <id> [--pr | --merge | --auto]
- Single phase. Pushes the epic branch, opens or updates a PR (or merges, with --merge).
- Idempotent. Running on an already-submitted epic updates the existing PR (gh_pr_create_or_update already does this).
- --pr (default): push and open PR
- --merge: do a working-tree merge of the epic into main, push main
- --auto: merge when clean, fall back to PR when the merge would conflict
- Does NOT delete the branch. Does NOT delete the worktree. Submit is about getting the work into main, nothing else.
- If --merge fails (conflict), the command fails loudly and suggests --pr as the next step.

apm epic close <id> [--force]
- Single phase. Deletes the local epic branch and removes the epic's worktree (if one exists). Optionally pushes the branch deletion to origin (git push origin --delete) — spec-writer to confirm whether this is default-on or behind a flag.
- Safety: if the epic branch has commits not present in main (regular ancestor check OR squash-merge check via the shared helper, both via origin/main preferred), refuse the close and print: epic has N commit(s) not yet in main. Use --force to confirm deletion (commits will be lost).
- --force: skip the safety check. Delete unconditionally. The supervisor's escape hatch for abandoning unsubmitted work.
- Close is irreversible (the branch is gone). The supervisor must intend it.

apm sync (additions)
- After the existing ticket-merge pass, add a second pass that scans local epic branches and prints up to two hint sections:
  1. Epics ready to submit: epic's derived state is done (all tickets terminal) but the branch is not yet merged into origin/main. Output: 'Epics ready to submit (apm epic submit <id>):' then a list.
  2. Epics ready to close: epic branch is merged into origin/main (use the shared squash-aware helper). Output: 'Epics ready to close (apm epic close <id>):' then a list.
- These are HINTS only. sync prints them and exits. It does not prompt to act on them. (Submit and close are real git mutations the supervisor must intend.)
- The detection uses the shared squash-merge helper (see below).

apm clean --epics is REMOVED. The --epics flag is dropped from apm clean. Bulk-clean was a stopgap; sync's hints + intentional apm epic close replace it. apm clean continues to handle ticket worktree cleanup as today.

NEW SHARED HELPER: apm-core/src/git_util.rs

Extract sync's existing squash-merge detection into a pub fn is_branch_content_merged(root, default_branch, branch) -> Result<bool>. Algorithm mirrors squash_merged today: compute merge_base, compare branch_tip; if equal return true; otherwise synthesize a virtual squash commit via git commit-tree branch carat braces tree -p merge_base, then git cherry default_branch virtual_squash and check for a leading dash. Prefer origin slash default_branch over local when the remote ref exists (mirrors merged_into_main's preference).

CONSUMERS:
- sync uses the helper in both the ticket-merge pass (already does, via squash_merged) and the new epic-detection pass.
- apm epic close uses the helper for its unmerged-work safety check.
- apm epic submit does not need the helper (it always pushes; gh handles the no-commits case).

BEHAVIORAL BREAK / MIGRATION:

This renames a public command (apm epic close changes meaning) and removes a flag (apm clean --epics). External scripts that invoke either will break. The fix is to update help text and the README to clearly describe the new model. Anyone who scripted today's apm epic close to mean push-then-clean must split it into apm epic submit (first run) + apm epic close (after merge).

OUT OF SCOPE:
- merge_failed-equivalent state for epics (today there is no epic state machine; the merge-conflict path is just an error message). If we add an epic state machine later, that is a separate concern.
- Adding tickets to an epic after submission. This already works naturally — adding a ticket pushes more commits to the epic branch, which gh auto-updates onto the open PR. No code change needed; just document.
- apm-server / apm-ui changes beyond surface-area renaming if any UI references the old close-name.
- Replacing the bulk-close path. If a future need for apm epic close --all-merged emerges, file then. For now: no bulk option.
- 0e55807c is superseded by this ticket. The squash-merge helper extraction is part of this scope; the worktree-cleanup behavior is part of apm epic close's new definition.
- dc2b08db (apm move worktree side-effect) is unrelated and unaffected.

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- apm epic submit on an epic with no PR pushes the branch and opens a PR. Output names the PR URL.
- apm epic submit on an epic with an open PR updates the PR (no new PR created). Output names the existing PR URL.
- apm epic submit --merge on an epic that would merge cleanly does the merge and pushes main. apm epic submit --merge on an epic that would conflict fails with a clear message suggesting --pr.
- apm epic submit --auto behaves like --merge when clean, --pr when conflicted.
- apm epic close on an epic whose branch is merged into origin/main (regular or squash) deletes the branch and removes the worktree.
- apm epic close on an epic with unmerged commits ahead of origin/main refuses, prints the ahead-count, and suggests --force.
- apm epic close --force deletes the branch and removes the worktree unconditionally.
- apm sync prints an Epics ready to submit section listing epics whose state is done and branch is not yet merged.
- apm sync prints an Epics ready to close section listing epics whose branch is merged into origin/main.
- apm sync does not prompt for any epic action; it only prints hints.
- apm clean has no --epics flag. apm clean --epics fails with an error suggesting apm epic close.
- A new public function apm_core::git_util::is_branch_content_merged exists and is used by sync and apm epic close.
- Unit tests for is_branch_content_merged: regular merge returns true; squash merge returns true; unmerged branch returns false; missing remote ref falls back to local.
- Integration test: end-to-end submit-then-close-after-merge flow for an empty epic and a populated epic.
- Integration test: sync hints appear after a PR merge and disappear after apm epic close.
- Help text for apm epic clearly documents submit vs close as two separate phases.
- README and any docs that reference today's apm epic close are updated.

REFERENCES:
- apm/src/cmd/epic.rs (run_close, run_epic_clean) — existing logic to refactor
- apm-core/src/git_util.rs::squash_merged (around line 217) — algorithm to extract
- apm-core/src/git_util.rs::merged_into_main (around line 102) — origin-preference pattern to mirror
- apm/src/cmd/clean.rs — remove the --epics branch
- apm/src/cmd/sync.rs and apm-core/src/sync.rs — add the epic-detection pass
- apm-core/src/worktree.rs — existing worktree-cleanup logic for apm epic close to reuse
- Background: syn project epic 72294403 hit squash-merge invisibility; design discussion in conversation history
- Supersedes: 0e55807c (which covered the helper extraction and worktree-cleanup parts of this work)

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
| 2026-06-02T06:05Z | — | new | philippepascal |
| 2026-06-02T06:07Z | new | groomed | philippepascal |
| 2026-06-02T06:11Z | groomed | in_design | philippepascal |
