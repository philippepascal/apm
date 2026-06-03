+++
id = "7c5cc82a"
title = "apm clean --branches: batch remote branch deletions into a single push"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5cc82a-apm-clean-branches-batch-remote-branch-d"
created_at = "2026-06-03T03:03:55.652009Z"
updated_at = "2026-06-03T06:46:04.593572Z"
+++

## Spec

### Problem

`apm clean --branches` deletes remote ticket branches by calling `git push origin --delete <branch>` once per candidate, serially. Each push incurs a full connection setup (SSH or HTTPS handshake), a local pre-push hook invocation, a remote pre-receive/post-receive cycle, and a network round-trip for the acknowledgement. With N branches, total wall-clock cost is N × that overhead. In projects where hundreds of ticket branches accumulate across epics, this makes `apm clean` take minutes per session.

Git natively supports deleting multiple refs in a single push: `git push origin --delete refs/heads/A refs/heads/B refs/heads/C ...` collapses the cost to a single connection, single hook cycle, and single round-trip regardless of N. The fix is to collect all remote-eligible branches across the candidate loop and issue one batched push after the loop, rather than one push inside the loop.

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
| 2026-06-03T06:46Z | groomed | in_design | philippepascal |