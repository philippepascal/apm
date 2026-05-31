+++
id = "1469175e"
title = "apm refresh-epic --merge: push after local merge so downstream sees the refresh"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1469175e-apm-refresh-epic-merge-push-after-local-"
created_at = "2026-05-31T03:26:11.802159Z"
updated_at = "2026-05-31T03:26:11.802159Z"
+++

## Spec

### Problem

BUG: apm refresh-epic --merge merges the default branch into the epic worktree locally but does not push. Downstream tickets dispatched via apm start after the merge read stale content because the dispatch path prefers origin over local refs.

EVIDENCE:

apm/src/cmd/epic.rs lines 182-202 (the --merge path):
- Resolves the epic worktree
- Calls merge_ref(epic_wt_path, default_branch, ...) — merges main into the worktree
- On success, prints messages and returns
- NO push step

Compare apm/src/cmd/epic.rs lines 203-225 (the --pr path), which DOES call push_branch_tracking before opening the PR. The two paths are asymmetric.

apm-core/src/start.rs lines 454-458 (dispatch-time merge for any apm start):
- Computes merge_base as ticket.frontmatter.target_branch or default_branch
- Calls remote_branch_tip(merge_base) — if the origin ref exists, prefers origin
- Falls back to local only when origin is missing

The combination: after a local-only --merge, origin/epic/<id>-... is unchanged. A subsequent apm start <T> on a ticket in the epic merges origin (stale), not the freshly-refreshed local epic. The refresh is silently useless for any ticket dispatched until the supervisor manually pushes.

WHAT WENT WRONG IN PRACTICE (syn project just hit this):
1. Supervisor ran apm refresh-epic <epic-id> --merge. Local epic branch advanced.
2. Supervisor or a worker started a new ticket in the epic.
3. apm start dispatched on origin/epic/<id> (the pre-refresh tip).
4. The worker received content that did not reflect the just-completed refresh.

FIX (suggested; spec-writer to refine):

apm refresh-epic --merge should push the refreshed epic branch to origin after the local merge succeeds. Three behaviour options for the spec-writer to choose between:

(A) Default-yes prompt. After a successful local merge, prompt push refreshed epic to origin? [Y/n]. Pressing enter pushes; n skips with a clear warning that downstream apm start invocations will read stale origin content until the supervisor pushes manually.

(B) Two flags. --push and --no-push. --push always pushes after merge (no prompt). --no-push always skips push (no prompt). Default behaviour without either flag: prompt as in option A. This adds CLI knobs but supports both scripted and interactive use.

(C) Always push. The merge represents an explicit supervisor decision to integrate; pushing is the logical completion. No flag or prompt. The cost is loss of the local-only testing path; the benefit is the inconsistency goes away.

Recommendation: B (two flags + default prompt). Mirrors how apm sync handles push decisions and gives the supervisor full control.

OUT OF SCOPE:
- The broader cascade-into-ticket-branches behaviour for apm refresh-epic. That is a separate, larger concern about in-flight workers seeing fresh code; it should be filed as its own ticket.
- Changes to the dispatch-time merge logic in apm-core/src/start.rs. The current 'prefer origin' behaviour is consistent with the rest of the codebase (merged_into_main also keys off origin); changing it is a separate design decision.

TESTS:
- After apm refresh-epic <id> --merge --push, origin/<epic-branch> points at the post-merge tip.
- After apm refresh-epic <id> --merge --no-push, origin/<epic-branch> is unchanged; a warning is printed.
- Interactive: apm refresh-epic <id> --merge prompts push refreshed epic to origin? [Y/n]; default Y pushes; n skips with warning.
- Existing --pr path is unchanged.
- Existing default (no --merge, no --pr) behaviour is unchanged.

REFERENCES:
- apm/src/cmd/epic.rs::run_refresh_epic (the --merge path at lines 182-202; the --pr path at 203-225)
- apm-core/src/start.rs::run lines 454-458 (the dispatch-time merge that prefers origin)
- apm-core/src/git.rs::push_branch_tracking (already used by --pr)
- apm-core/src/git_util.rs::remote_branch_tip (the function that drives the origin preference)

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
| 2026-05-31T03:26Z | — | new | philippepascal |
