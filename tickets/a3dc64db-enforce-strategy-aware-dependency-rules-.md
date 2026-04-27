+++
id = "a3dc64db"
title = "Enforce strategy-aware dependency rules at every write site"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a3dc64db-enforce-strategy-aware-dependency-rules-"
created_at = "2026-04-27T20:28:18.110435Z"
updated_at = "2026-04-27T20:42:21.104980Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

`apm new --depends-on` and `apm start` currently accept dependencies regardless of the configured completion strategy. The spec at `docs/strategy-and-dependencies.md` (section 'Dependency rules per strategy') defines when dependencies compose safely:

- pr_or_epic_merge: ticket and all deps must share an epic
- merge: ticket and all deps must share target_branch (same epic, or all on default)
- pr / none: --depends-on is rejected outright

Implement the rule check at both creation time (`apm-core/src/ticket/ticket_util.rs::create`, around the `depends_on` parameter) and at start time (`apm-core/src/start.rs`, before transitioning to `in_progress`). Reject violations with a clear message naming the offending dep IDs and the rule that was broken.

The rule depends on the completion strategy of the `in_progress → implemented` transition, read from `workflow.toml`. The strategy determination logic should live in a single helper that the start path, the new path, and `apm validate` (a separate ticket) all share.

See docs/strategy-and-dependencies.md, section 'Dependency rules per strategy'.

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
| 2026-04-27T20:28Z | — | new | philippepascal |