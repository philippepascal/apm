+++
id = "fb30010b"
title = "wrong message when epic submit doesn't need to merge anything"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fb30010b-wrong-message-when-epic-submit-doesn-t-n"
created_at = "2026-09-01T00:59:43.049321Z"
updated_at = "2026-09-01T01:04:15.158538Z"
+++

## Spec

### Problem

`apm epic submit` produces misleading, actionable-free errors when the epic
branch has no commits beyond the default branch — e.g. because its tickets
already landed via another path, or the branch was already merged manually.

With the default `--pr` mode, `run_submit` (`apm/src/cmd/epic.rs`) shells out
straight to `gh pr create`. When there is nothing to submit, `gh` fails with
a raw GraphQL error that gets surfaced verbatim: `Error: gh pr create failed:
... GraphQL: No commits between main and epic/... (createPullRequest)`. With
`--merge`, `apm_core::git_util::merge_ref` treats "Already up to date" (a
successful no-op `git merge`) identically to a real conflict — both paths
return `None` — so `run_submit` reports `Error: merge conflict — resolve
manually after checking out main, or use --pr to open a PR instead`, even
though there is no conflict: `git merge` itself would report "Already up to
date."

Both messages send the user chasing a nonexistent problem instead of telling
them the real situation: there is nothing new to submit, and the epic branch
can most likely just be closed with `apm epic close`.

### Acceptance criteria

- [ ] `apm epic submit <id>` (default `--pr` behaviour) on an epic branch with zero commits ahead of the default branch prints a "nothing to submit" message and exits 0, without invoking `gh pr create`
- [ ] `apm epic submit <id> --merge` on an epic branch with zero commits ahead of the default branch prints the same "nothing to submit" message and exits 0, without attempting `git merge` and without any "merge conflict" text
- [ ] `apm epic submit <id> --auto` on an epic branch with zero commits ahead of the default branch also short-circuits with the "nothing to submit" message and exits 0, without attempting a merge or a PR
- [ ] The "nothing to submit" message names the default branch and suggests `apm epic close <id>` as the next step
- [ ] `apm epic submit <id> --merge` on an epic branch that has unmerged commits and a genuine merge conflict still reports `merge conflict — resolve manually ...` unchanged
- [ ] `apm epic submit <id> --merge` on an epic branch with unmerged, cleanly-mergeable commits still merges into the default branch and reports success unchanged
- [ ] `apm epic submit <id>` (default `--pr`) on an epic branch with unmerged commits still opens or updates a PR unchanged

### Out of scope

- Fixing the general conflation in `apm_core::git_util::merge_ref`, where a `None` return means either "already up to date" or "real conflict" — this ticket only needs `apm epic submit` to stop reaching that call at all when there's nothing to merge. Other callers (`apm-core/src/start.rs`, `apm epic refresh` in `apm/src/cmd/epic.rs`) are unaffected: `run_refresh_epic` already guards this exact case with its own upfront `ahead == 0` check before calling `merge_ref`.
- Adding quiescence or live-worker checks to `apm epic submit` — only `apm epic close` has those today.
- Any change to `apm epic close` behaviour.
- Auto-closing the epic when there is nothing left to submit — the fix only suggests `apm epic close <id>` as the next step, it does not run it.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-09-01T00:59Z | — | new | philippepascal |
| 2026-09-01T01:04Z | new | groomed | philippepascal |
| 2026-09-01T01:04Z | groomed | in_design | philippepascal |