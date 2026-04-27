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

`apm new --depends-on` and `apm set <id> depends_on …` currently accept dependencies regardless of the configured completion strategy. The spec at `docs/strategy-and-dependencies.md` (section 'Dependency rules per strategy') defines when dependencies compose safely:

- pr_or_epic_merge: ticket and all deps must share an epic
- merge: ticket and all deps must share target_branch (same epic, or all on default)
- pr / none: --depends-on is rejected outright

Implement the rule check at every site where `depends_on` is written:

- `apm new` — `apm-core/src/ticket/ticket_util.rs::create`, around the `depends_on` parameter
- `apm set <id> depends_on <ids>` — wherever the `set` subcommand handles the `depends_on` field

Reject violations with a clear message naming the offending dep IDs and the rule that was broken.

Re-validating at `apm start` is **not** required: the hash-trip / `apm validate` mechanism (separate ticket) catches the post-hoc case where a previously-valid setup becomes invalid after a config change. Validating at every write site plus the hash-trip is sufficient and avoids redundant checks on a hot path.

The strategy-determination logic (read from `workflow.toml` to find the `in_progress → implemented` transition's `completion` field) and the per-strategy rule check should live in a single helper that the write paths and `apm validate` (separate ticket) all share.

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