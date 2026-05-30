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
| 2026-05-30T02:17Z | — | new | philippepascal |
| 2026-05-30T02:18Z | new | groomed | philippepascal |
| 2026-05-30T02:32Z | groomed | in_design | philippepascal |