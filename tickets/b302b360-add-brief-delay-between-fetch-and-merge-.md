+++
id = "b302b360"
title = "Add brief delay between fetch and merge in apm start to reduce fetch-race window"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b302b360-add-brief-delay-between-fetch-and-merge-"
created_at = "2026-05-03T08:07:25.634157Z"
updated_at = "2026-05-03T19:01:49.415012Z"
+++

## Spec

### Problem

When apm start fetches origin/main before merging into the ticket branch, a narrow race window exists: if a previous ticket was merged to origin within ~30 seconds before apm start fires, the fetch may retrieve a stale snapshot and the merge silently operates on old content. Observed in 6095305a (f06272f1 merged at 12:21:51, apm start at 12:22:14 — 23-second window). The stale merge succeeded, the worker built on the old start.rs base, and the subsequent apm state implemented merge conflicted with f06272f1's changes. A short deterministic sleep (e.g. 1-2 seconds) between fetch and merge gives the remote propagation window time to settle, reducing the probability of this race without requiring retries or polling.

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
| 2026-05-03T08:07Z | — | new | philippepascal |
| 2026-05-03T19:01Z | new | groomed | philippepascal |
