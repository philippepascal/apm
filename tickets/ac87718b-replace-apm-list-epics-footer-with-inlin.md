+++
id = "ac87718b"
title = "Replace apm list epics footer with inline ↓ marker on tickets whose epic is behind main"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ac87718b-replace-apm-list-epics-footer-with-inlin"
created_at = "2026-05-30T02:17:52.780155Z"
updated_at = "2026-05-30T02:32:24.166390Z"
+++

## Spec

### Problem

The `apm list` output includes an `epics:` footer block (introduced by ticket 7a76dd16) that lists each stale epic with its commit count and conflict label. This adds vertical noise to the most-used triage command and forces the user to mentally cross-reference the footer against ticket rows above to determine which specific tickets are on stale epics.

`apm epic list` already surfaces per-epic freshness in full detail. In `apm list`, the actionable signal is *which tickets* are on stale epics — the commit count and conflict label are secondary. Replacing the footer with a bare `↓` marker inline on each affected ticket row delivers that signal at the point of relevance and removes the footer entirely.

### Acceptance criteria

- [ ] `apm list` output never contains an `epics:` section header or footer block under any circumstances.
- [ ] A ticket whose `target_branch` starts with `epic/` and whose epic branch is behind `main` shows `↓` appended to the epic ID in the base column (e.g. `ab12cd34↓`).
- [ ] A ticket whose `target_branch` starts with `epic/` and whose epic branch is up to date with `main` shows no `↓` in its row.
- [ ] A ticket with no `target_branch` (main-scoped) shows no `↓` in its row.
- [ ] A ticket whose `target_branch` does not start with `epic/` shows no `↓` in its row.
- [ ] When two tickets share the same stale epic, both rows show `↓` (epic freshness is deduped per epic ID, not computed per ticket row).
- [ ] The `↓` marker appears unchanged in piped (non-TTY) output — no ANSI codes surround it.
- [ ] `apm epic list` output is unchanged by this ticket.

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
| 2026-05-30T02:17Z | — | new | philippepascal |
| 2026-05-30T02:18Z | new | groomed | philippepascal |
| 2026-05-30T02:32Z | groomed | in_design | philippepascal |