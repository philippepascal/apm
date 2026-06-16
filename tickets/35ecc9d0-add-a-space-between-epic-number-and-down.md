+++
id = "35ecc9d0"
title = "add a space between epic number and down arrow to make the down arrow more noticeable"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/35ecc9d0-add-a-space-between-epic-number-and-down"
created_at = "2026-06-16T18:06:38.591826Z"
updated_at = "2026-06-16T18:11:42.499624Z"
+++

## Spec

### Problem

When `apm list` displays a ticket whose epic branch is ahead of the default branch (i.e., the epic needs a rebase), it appends a down-arrow indicator to the epic ID in the base column — for example `a1b2↓`. Because there is no space between the ID and the arrow, the indicator blends visually with the hex digits and is easy to miss at a glance.

Adding a single space between the ID and the arrow (`a1b2 ↓`) makes the indicator stand out without changing its meaning or layout significantly.

### Acceptance criteria

- [ ] `apm list` output for a ticket in a stale epic (epic branch ahead of default branch) shows a space before the down arrow in the base column: `<epic-id> ↓`
- [ ] `apm list` output for a ticket in a non-stale epic shows only the epic ID with no trailing space or arrow
- [ ] `apm list` output for a ticket with no epic (base column shows the default branch) is unchanged

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
| 2026-06-16T18:06Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:11Z | groomed | in_design | philippepascal |