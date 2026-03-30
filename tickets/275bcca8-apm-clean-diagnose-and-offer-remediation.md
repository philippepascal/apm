+++
id = "275bcca8"
title = "apm clean: diagnose and offer remediation for dirty worktrees"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/275bcca8-apm-clean-diagnose-and-offer-remediation"
created_at = "2026-03-30T18:12:35.205840Z"
updated_at = "2026-03-30T19:16:53.587019Z"
+++

## Spec

### Problem

When `apm clean` encounters a worktree with uncommitted changes, it silently skips it with a one-line warning. The user gets no actionable information: not what the files are, not whether they matter, not what to do.

In practice, dirty worktrees fall into a few distinct categories that each warrant a different response:

- **Untracked temp files** (`pr-body.md`, `.apm-worker.pid`, `ac.txt`, etc.) — leftover worker artifacts that are safe to delete. The worktree is effectively clean for the purposes of branch removal.
- **Stale PID files / log files** (`.apm-worker.pid`, `.apm-worker.log`) — the process is gone; the file is noise.
- **Untracked user files** — possibly intentional; user should decide.
- **Modified tracked files** — real uncommitted work; definitely needs user attention before cleaning.

The current behaviour conflates all of these into "has uncommitted changes — skipping", which:
1. Doesn't distinguish safe-to-clean from risky cases
2. Gives no remediation path
3. Forces the user to manually inspect and clean each worktree before `apm clean` will proceed

The desired behaviour: `apm clean` diagnoses each blocked worktree, explains what it found (categorised), proposes a concrete action (e.g. "remove 3 untracked temp files"), and asks the user to confirm or skip. Modified tracked files are always left for the user to handle manually.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T18:12Z | — | new | philippepascal |
| 2026-03-30T19:16Z | new | in_design | philippepascal |
