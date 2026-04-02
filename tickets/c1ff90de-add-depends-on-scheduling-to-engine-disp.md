+++
id = "c1ff90de"
title = "Add depends_on scheduling to engine dispatch loop"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/c1ff90de-add-depends-on-scheduling-to-engine-disp"
created_at = "2026-04-01T21:55:02.787625Z"
updated_at = "2026-04-02T00:43:15.939908Z"
+++

## Spec

### Problem

Once `depends_on` is stored in ticket frontmatter (ticket d877bd37), the engine dispatch loop must honour it. Currently the loop picks the highest-priority ready ticket and dispatches it immediately — it has no concept of blocked tickets.

The full design is in `docs/epics.md` (§ depends_on scheduling — Engine loop change). Before dispatching a candidate, the loop must check each entry in `depends_on`: if any referenced ticket is not yet `implemented` or later, the candidate is skipped. The check is config-driven: "implemented or later" is determined by position in the workflow states list or by `terminal = true`, not hardcoded state names. Unknown dep IDs are treated as non-blocking.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
