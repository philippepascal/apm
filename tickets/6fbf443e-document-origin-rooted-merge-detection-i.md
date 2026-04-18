+++
id = "6fbf443e"
title = "Document origin-rooted merge detection in apm sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6fbf443e-document-origin-rooted-merge-detection-i"
created_at = "2026-04-18T02:21:39.386683Z"
updated_at = "2026-04-18T02:26:42.225327Z"
+++

## Spec

### Problem

`apm sync`'s "merged → closeable" detection is rooted at `origin/<default_branch>`, not local `<default_branch>`. This is intentional: a ticket is considered "done" only when its merge is visible to the rest of the team, which means the merge commit must be on origin.

Current state: this design choice is not documented anywhere. Users (and agents) hit it when they merge locally but haven't pushed, see `apm sync` report "N commits ahead — run `git push` when ready" without understanding the connection, and then wonder why merged tickets aren't offered for closing until after `git push` runs.

Concretely, `apm-core/src/git_util.rs::merged_into_main` (line ~85) begins with `refs/remotes/origin/<default>` and uses `origin/<default>` as the merge oracle; local `<default>` is only a fallback when origin cannot be resolved at all. This is the right behavior — it prevents a user with an out-of-sync local branch from closing tickets the team can't yet see — but the reasoning needs to be captured.

Where to document: `docs/commands.md` already covers per-command behavior; the `apm sync` section should explain the `origin/<default>`-rooted detection explicitly. A companion mention in `.apm/agents.md` is worth considering so that agents understand why a ticket they merged locally isn't showing up as closeable until main is pushed.

Trigger: user hit this on 2026-04-17 after merging `37323beb` locally; `apm sync` reported "main is ahead of origin/main by 16 commits — run `git push` when ready" but did not offer to close the merged ticket. After `git push`, a subsequent `apm sync` immediately offered to close it. The surprise was understandable because the relationship between "push main" and "close detection" is not obvious from any documentation.

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
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:26Z | groomed | in_design | philippepascal |
