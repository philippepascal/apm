+++
id = "c2168aea"
title = "Remove accepted state and simplify apm sync to hardcode merged-PR-to-closed"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "70351"
branch = "ticket/c2168aea-remove-accepted-state-and-simplify-apm-s"
created_at = "2026-04-01T20:26:50.809264Z"
updated_at = "2026-04-01T20:29:57.337864Z"
+++

## Spec

### Problem

The APM workflow has an unnecessary intermediate state, **`accepted`**, that sits between `implemented` and `closed`. Its sole purpose is to be an automatic waypoint: `apm sync` detects merged branches, transitions `implemented` tickets to `accepted`, then on the next sync run transitions `accepted` tickets to `closed`. This two-step dance provides no value — the PR merge is already the acceptance signal. Tickets should go from `implemented` directly to `closed` when their branch is merged.

Compounding this, `apm sync` decides which tickets to inspect by reading the `completion` field in each state's transition config. This couples the sync behaviour to config structure in a fragile way. The correct rule is simpler and needs no config: scan every non-terminal ticket, check whether its branch's PR has been merged on GitHub, and if so close it immediately.

Both changes together eliminate a redundant state, shorten the closing cycle from two sync runs to one, and remove the config dependency from the sync scan loop.

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
| 2026-04-01T20:26Z | — | new | apm |
| 2026-04-01T20:29Z | new | in_design | philippepascal |