+++
id = "6e3f9e91"
title = "Add global max_workers_per_epic config; remove per-epic override"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6e3f9e91-add-global-max-workers-per-epic-config-r"
created_at = "2026-04-27T20:28:07.069581Z"
updated_at = "2026-04-27T20:47:50.085386Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

Per-epic concurrency is currently configurable via `apm epic set <id> max_workers <n>` (see `apm-core/src/epic.rs`, the `epic set` subcommand). The spec at `docs/strategy-and-dependencies.md` (section 'Epic concurrency') makes the parallelism unit the epic itself: each epic holds at most one active worker, and users gain parallelism by creating more epics rather than tuning concurrency within one.

Add a global `max_workers_per_epic` setting under a new section in `.apm/config.toml` (or the project apm.toml schema), default 1. Enforce in the dispatch path (`pick_next` / `apm start --next --spawn`) so a ticket in epic E is not picked while another worker is already active in E.

Remove the per-epic override:
- Remove the `max_workers` field from the epic frontmatter
- Remove `apm epic set <id> max_workers ...` (the `set` subcommand should accept only `owner` after this change)
- Update tests

See docs/strategy-and-dependencies.md, section 'Epic concurrency'.

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
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:47Z | groomed | in_design | philippepascal |
