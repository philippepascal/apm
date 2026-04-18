+++
id = "6fbf443e"
title = "Document origin-rooted merge detection in apm sync"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6fbf443e-document-origin-rooted-merge-detection-i"
created_at = "2026-04-18T02:21:39.386683Z"
updated_at = "2026-04-18T07:38:46.066407Z"
+++

## Spec

### Problem

`apm sync`'s "merged → closeable" detection is rooted at `origin/<default_branch>`, not local `<default_branch>`. This is intentional: a ticket is considered "done" only when its merge is visible to the rest of the team, which means the merge commit must be on origin.

Current state: this design choice is not documented anywhere. Users (and agents) hit it when they merge locally but haven't pushed, see `apm sync` report "N commits ahead — run `git push` when ready" without understanding the connection, and then wonder why merged tickets aren't offered for closing until after `git push` runs.

Concretely, `apm-core/src/git_util.rs::merged_into_main` (line ~85) begins with `refs/remotes/origin/<default>` and uses `origin/<default>` as the merge oracle; local `<default>` is only a fallback when origin cannot be resolved at all. This is the right behavior — it prevents a user with an out-of-sync local branch from closing tickets the team can't yet see — but the reasoning needs to be captured.

Where to document: `docs/commands.md` already covers per-command behavior; the `apm sync` section should explain the `origin/<default>`-rooted detection explicitly. A companion mention in `.apm/agents.md` is worth considering so that agents understand why a ticket they merged locally isn't showing up as closeable until main is pushed.

Trigger: user hit this on 2026-04-17 after merging `37323beb` locally; `apm sync` reported "main is ahead of origin/main by 16 commits — run `git push` when ready" but did not offer to close the merged ticket. After `git push`, a subsequent `apm sync` immediately offered to close it. The surprise was understandable because the relationship between "push main" and "close detection" is not obvious from any documentation.

### Acceptance criteria

- [x] `docs/commands.md` `apm sync` Description explicitly states that merge detection is rooted at `origin/<default>`, not local `<default>`
- [x] `docs/commands.md` `apm sync` Description states that a ticket merged locally but not yet pushed to origin will not be offered for closure until after `git push`
- [x] `docs/commands.md` `apm sync` Git internals table row for `git branch -r --merged origin/<default>` clarifies that the remote ref is used intentionally (not a local branch)
- [x] `.apm/agents.md` Startup section includes a note that `apm sync` closes tickets only once their merge is visible on `origin/<default>`, and that `git push` on the default branch is a prerequisite for closure detection

### Out of scope

- Changing the behavior of `merged_into_main` or `apm sync` — detection logic stays as-is
- Changing the CLI output messages produced by `apm sync` (e.g. the "N commits ahead" warning)
- Adding automated tests for documentation content

### Approach

Two files change; no code changes.

**1. `docs/commands.md` — `apm sync` Description (line ~583)**

After the sentence "detects ticket branches that have been merged (including squash-merges) into the default branch, and closes those tickets", append a new paragraph:

> **Merge detection is rooted at `origin/<default-branch>`, not at local `<default-branch>`.** A ticket branch is considered merged only when its merge commit is visible on the remote tracking ref. If you merge a ticket branch into `main` locally but have not yet pushed, `apm sync` will report that main is ahead of origin/main and will not offer to close the ticket. Run `git push` on the default branch first; the next `apm sync` will then detect the merge and offer closure. This is intentional: it prevents tickets from being closed before the team can see the merge.

**2. `docs/commands.md` — `apm sync` Git internals table (line ~611)**

Update the Why comment for the `git branch -r --merged origin/<default>` row from:

> Find branches merged into the default branch via a regular merge

to:

> Find remote ticket branches merged into `origin/<default>` (intentional: local `<default>` is not checked — a merge must be pushed before it counts)

**3. `.apm/agents.md` — Startup section (after step 1 "apm sync")**

Add an indented note beneath step 1:

> **Note:** `apm sync` detects merges via `origin/<default-branch>`. A ticket merged into your local default branch but not yet pushed will not appear as closeable. If you expect a ticket to be offered for closure after merging locally, run `git push` on the default branch first, then re-run `apm sync`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:26Z | groomed | in_design | philippepascal |
| 2026-04-18T02:28Z | in_design | specd | claude-0418-0226-6a70 |
| 2026-04-18T06:51Z | specd | ready | apm |
| 2026-04-18T06:52Z | ready | in_progress | philippepascal |
| 2026-04-18T06:53Z | in_progress | implemented | claude-0418-0652-abd0 |
| 2026-04-18T07:38Z | implemented | closed | apm-sync |
