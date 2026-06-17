+++
id = "b9aed386"
title = "Propagate terminal state to main for already-merged epic tickets"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b9aed386-propagate-terminal-state-to-main-for-alr"
created_at = "2026-06-17T00:19:57.975332Z"
updated_at = "2026-06-17T00:19:57.975332Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, its terminal-state transitions (close, and other terminal states) are committed to two places: the ticket own branch (authoritative, hard write) and target_branch which is the epic branch (best-effort, soft write). See close() at apm-core/src/ticket/ticket_util.rs:355-362 and transition() at apm-core/src/state.rs:178-193. After the epic code is merged to main via apm epic submit, any later terminal transitions (for example apm sync closing implemented tickets) append ticket-file-only commits to the epic branch but never to main. For an epic ticket target_branch is the epic branch, so close() never writes the closed state to main (contrast: a non-epic ticket has target_branch None so it defaults to main and closes directly onto main).

Consequence: a closed epic ticket has its final state only on the ticket branch and the epic branch. The ticket file on main stays frozen at the epic-submit-time state (for example implemented). The only mechanism that reconciles this to main is the manual apm archive command, which reads the ticket branch (apm-core/src/archive.rs:71) and writes the closed content into the archive directory on main (archive.rs:143). That reconciliation is manual, separate, and order-dependent: it only works while the ticket branch still exists. If apm clean-branches prunes the ticket branch before apm archive runs, the closed state is lost from everywhere reachable and main shows the stale state permanently. apm epic close deleting the epic branch makes this easier to hit because closing the epic becomes frictionless.

Proposed root-cause fix (supersedes ticket 57423ff5, which only relaxed the apm epic close guard and treated the symptom): when a ticket target_branch is an epic that is already merged to main, route the terminal-state commit to the default branch (main) instead of, or in addition to, the dead epic branch. Reuse the existing detection git_util::content_merged_into_main(root, main_ref, branch, tickets_dir) to decide already-merged. This is proven machinery: non-epic tickets already close straight to main via target = target_branch.unwrap_or(default_branch) at ticket_util.rs:358. Apply the choice in both close() (ticket_util.rs:358-362) and the transition() terminal path (state.rs:187-193).

Why this over the alternative considered (apm sync doing an internal apm epic submit): epic submit defaults to opening a PR and only merges with --merge or --auto, requires the main worktree to be checked out on the default branch (it bails otherwise), creates a fresh merge commit on main for every sync that touches an epic ticket, and inherits the whole merge-failure surface inside what should be a safe refresh. The transition-layer write is a small single-file commit that works in PR and merge workflows, has no conflict surface, needs no particular checkout, and covers manual apm state as well as sync.

Acceptance should include an integration test proving that closing an epic ticket whose epic code is already merged lands the closed state on main, and that apm epic close then succeeds under the original is_branch_content_merged guard. Consider whether the redundant epic-branch write should be dropped entirely once the epic is merged. Out of scope: changing apm epic submit; changing apm archive.

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
| 2026-06-17T00:19Z | — | new | philippepascal |
