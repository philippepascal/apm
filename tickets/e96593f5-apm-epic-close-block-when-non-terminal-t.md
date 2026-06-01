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

- [ ] `apm epic close <id>` succeeds unchanged when every ticket in the epic is in `closed` state.
- [ ] `apm epic close <id>` exits non-zero and prints a table of non-terminal tickets (id, state, title) when at least one ticket is non-terminal.
- [ ] The non-terminal bail message ends with `Re-run with --close-all to cascade close, or close them manually first.`
- [ ] `apm epic close <id> --close-all` exits non-zero and prints a table of offending tickets before closing anything when at least one ticket is in `blocked` or `question`.
- [ ] `apm epic close <id> --close-all` closes each non-terminal ticket and then closes the epic when all non-terminal tickets are in states other than `blocked`/`question`.
- [ ] `apm epic close <id> --close-all` prints `closing ticket #<id> ... done` for each ticket it closes.
- [ ] `apm epic close <id> --close-all` with a mix of `blocked` and closable tickets bails before modifying any ticket or the epic.
- [ ] The existing quiescence check (live workers, active coding states) still runs before the new non-terminal check in both paths.

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