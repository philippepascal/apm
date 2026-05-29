+++
id = "39b9c568"
title = "apm sync must not auto-close pre-implementation tickets"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/39b9c568-apm-sync-must-not-auto-close-pre-impleme"
created_at = "2026-05-29T00:18:15.128911Z"
updated_at = "2026-05-29T00:20:14.529754Z"
+++

## Spec

### Problem

BUG: apm sync auto-closes tickets in pre-implementation states (new/groomed/specd/question) when their branch's fork point reached main via an unrelated merge. Concretely: a side-note ticket (apm new --side-note creates a plain ticket in 'new' state whose branch only ever contains the ticket .md file, no implementation, no frontmatter marker) gets flagged as 'branch content merged' and closed by sync.

ROOT CAUSE: sync::detect Case 3 calls git_util::content_merged_into_main. For a branch with zero non-ticket-file commits, content_tip is None, and step 6's regular-merge sub-case fires: it sees merge_base is not on main's first-parent chain (because the branch's fork point was pulled into main via an epic's --no-ff merge commit's side parent) and returns true. But the branch's own commits were never merged into main. For a real ticket the implementation commit sits BELOW the merge-base (so it is in main); for an unmerged side-note the only real commit sits ABOVE the merge-base and was never merged. content_merged_into_main cannot cheaply distinguish these, so the fix belongs in sync, not in that function.

FIX: in sync::detect, the merge-close passes (Case 1 'branch merged' AND Case 3 'branch content merged') must skip tickets whose state is pre-implementation. A ticket that never reached 'implemented' has no completed work that could have merged, so any merge signal there is a git-topology artifact. Define pre-implementation as states that occur before any code is written: new, groomed, specd, question. Skip those in both Case 1 and Case 3 (do not push them as close candidates).

CRITICAL — NO BEHAVIOR CHANGE FOR ANY OTHER CASE: Case 2 (implemented ticket on main with branch gone) is unaffected. Case 4 (target_branch merge) is unaffected. Tickets in implemented (and any post-implementation state) must still be detected and closed exactly as today. The hint-generation pass is unaffected. The only change is that Case 1 and Case 3 no longer close tickets in new/groomed/specd/question. Existing tests for implemented-ticket detection must continue to pass unchanged.

DO NOT change content_merged_into_main, merged_into_main, or any git_util function. The fix is purely a state filter in sync::detect. Add an integration test: a side-note ticket (state new) on a branch whose fork point is in main via an epic --no-ff merge must NOT appear in close candidates and must NOT generate a hint; and a regression test confirming an implemented ticket on a content-merged branch is still closed.

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
| 2026-05-29T00:18Z | — | new | philippepascal |
| 2026-05-29T00:20Z | new | groomed | philippepascal |
| 2026-05-29T00:20Z | groomed | in_design | philippepascal |
