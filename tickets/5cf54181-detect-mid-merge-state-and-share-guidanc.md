+++
id = "5cf54181"
title = "Detect mid-merge state and share guidance strings"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5cf54181-detect-mid-merge-state-and-share-guidanc"
created_at = "2026-04-17T18:32:40.602264Z"
updated_at = "2026-04-17T18:34:29.146331Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
+++

## Spec

### Problem

Two supporting concerns shared across the other sync tickets:

1. **Mid-merge state is undetected.** If the user runs `apm sync` while the repo is in a mid-merge, mid-rebase, or mid-cherry-pick state (e.g. `.git/MERGE_HEAD` exists), sync's attempts to fast-forward or merge will compound the mess. Sync should detect this state at the top of the flow and bail with clear guidance ("finish or abort first").

2. **Guidance strings are scattered.** Tickets A and B both need copy-pasteable recovery instructions for scenarios sync cannot auto-handle (dirty-overlap FF, diverged main, diverged ticket/epic branch, mid-merge repo). Having these strings defined once in a small module keeps wording consistent and makes future tweaks single-point.

This ticket provides the mid-merge detection and the shared guidance-strings module that tickets A and B consume. It lands first in sequence but is small in scope.

See `/Users/philippepascal/Documents/apm/apm-sync-scenarios.md` — particularly the "Dirty-tree edge cases" and "Guidance copy" sections — for the full list of messages and their triggers. Implementers must add comments explaining when each guidance string fires.

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
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |