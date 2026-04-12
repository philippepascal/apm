+++
id = "061d0ac1"
title = "Add missing git helpers to git_util.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/061d0ac1-add-missing-git-helpers-to-git-util-rs"
created_at = "2026-04-12T17:29:22.472769Z"
updated_at = "2026-04-12T17:31:57.684129Z"
epic = "6062f74f"
target_branch = "epic/6062f74f-consolidate-git-operations-into-git-util"
+++

## Spec

### Problem

21 raw `Command::new("git")` calls are scattered across `clean.rs`, `epic.rs`, `init.rs`, `start.rs`, and `worktree.rs`, bypassing `git_util.rs` entirely. These modules construct git commands directly, duplicating argument patterns and spreading git implementation details throughout the codebase.

`git_util.rs` already defines a `run()` helper that centralises command construction, error formatting, and stdout capture — but nine behaviours are absent from its public API:

- detecting whether a worktree has uncommitted changes
- checking whether a local branch ref exists
- deleting a local branch (non-fatal)
- pruning a remote-tracking ref (silent)
- staging a list of files
- creating a commit from the working tree
- reading a git config key
- merging an arbitrary ref with output reporting
- checking whether a path is tracked by git

Because these helpers are missing, callers must either inline the git command or (in the case of `has_commits` and `fetch_branch`) re-implement helpers that already exist in `git_util.rs`.

This ticket adds the nine missing helpers. A separate ticket will update each caller to use them.

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
| 2026-04-12T17:29Z | — | new | philippepascal |
| 2026-04-12T17:30Z | new | groomed | apm |
| 2026-04-12T17:31Z | groomed | in_design | philippepascal |