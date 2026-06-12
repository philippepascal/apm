+++
id = "ba6da9cb"
title = "apm sync incorrectly mentions main instead of epic name in error message for missing merge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba6da9cb-apm-sync-incorrectly-mentions-main-inste"
created_at = "2026-06-09T21:47:31.578694Z"
updated_at = "2026-06-12T07:58:00.596811Z"
+++

## Spec

### Problem

When `apm sync` detects a ticket that appears to be in an `implemented`-equivalent state but whose branch has not been detected as merged, it emits a hint telling the user the ticket was not merged into `main`. This hint hardcodes the word "main" regardless of what the actual target branch is.

A ticket can target a branch other than the project default — most commonly an epic branch set via `target_branch` in the frontmatter. When such a ticket is unmerged, the hint is misleading: it points the user at `main` when the real merge target is, for example, `epic/abc-user-auth`. Even for tickets with no `target_branch`, the project default branch is configurable and may not be called `main`.

The fix is localised to the hint-generation block in `apm-core/src/sync.rs`.

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
| 2026-06-09T21:47Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T07:58Z | groomed | in_design | philippepascal |