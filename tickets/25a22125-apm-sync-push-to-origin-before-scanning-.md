+++
id = "25a22125"
title = "apm sync push to origin before scanning tickets. it might make more sense to push after the states have been changed."
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25a22125-apm-sync-push-to-origin-before-scanning-"
created_at = "2026-06-25T00:47:40.559751Z"
updated_at = "2026-06-25T06:41:56.823223Z"
+++

## Spec

### Problem

`apm sync` currently pushes locally-ahead ticket and epic branches to origin **before** it scans for tickets to auto-close. This means any close commits written by `sync::apply` (via `ticket::close`) are left sitting on local branches — they are not published to origin within the same sync run and only reach origin on the next `apm sync` invocation.

The correct order is: detect merge candidates, apply closures, then push. With that ordering, the push prompt covers every pending local commit in one shot — including close-state commits just written by the auto-close step — so origin stays current after a single `apm sync`. The reordering also applies to the default-branch push, which belongs at the end for the same reason.

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
| 2026-06-25T00:47Z | — | new | philippepascal |
| 2026-06-25T06:41Z | new | groomed | philippepascal |
| 2026-06-25T06:41Z | groomed | in_design | philippepascal |