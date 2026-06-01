+++
id = "e96593f5"
title = "apm epic close: block when non-terminal tickets exist; add --close-all to cascade"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e96593f5-apm-epic-close-block-when-non-terminal-t"
created_at = "2026-05-31T03:26:36.317944Z"
updated_at = "2026-06-01T02:57:17.529086Z"
+++

## Spec

### Problem

`apm epic close` runs a quiescence check (no live workers, no tickets currently in
an active coding state) but does not verify that all tickets in the epic have
reached a terminal state. A ticket in `specd`, `new`, `groomed`, `blocked`
(pre-implementation), `in_design` (no live worker), or `question` passes
quiescence fine, yet the epic closes around it: the epic branch is deleted or a
PR is opened, and those tickets are silently orphaned — still carrying the epic's
ID in their `epic` frontmatter field but with no managed path forward.

The fix is a second guard, separate from quiescence, that enforces a fully closed
epic before the branch is touched. Without `--close-all` the command bails and
tells the supervisor which tickets still need attention. With `--close-all` it
cascades a force-close over safe tickets, but refuses to silently swallow tickets
in `blocked` or `question`, which represent open questions that would lose their
context if closed without review.

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
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:57Z | groomed | in_design | philippepascal |