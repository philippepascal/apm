+++
id = "328b3c71"
title = "apm work --dry-run: show all candidates up to max-workers"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
agent = "61054"
branch = "ticket/328b3c71-apm-work-dry-run-show-all-candidates-up-"
created_at = "2026-03-30T16:31:01.147894Z"
updated_at = "2026-03-30T16:42:26.208092Z"
+++

## Spec

### Problem

Currently, `apm work --dry-run` shows only the single highest-priority ticket that the work loop would dispatch next. When a project has `max_concurrent > 1`, the work loop actually starts multiple workers in parallel — but the dry-run output gives no indication of which tickets those would be.

This makes dry-run nearly useless as a preview tool. Users who want to sanity-check what `apm work` would do before running it for real cannot see the full picture: they only see the first dispatch, not the second or third.

The fix is to have `--dry-run` show up to `max_concurrent` candidates in priority order — matching what the actual work loop would start.

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
| 2026-03-30T16:31Z | — | new | philippepascal |
| 2026-03-30T16:39Z | new | in_design | philippepascal |