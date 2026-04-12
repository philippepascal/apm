+++
id = "6c25c06a"
title = "Apply util.rs helpers across command files"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6c25c06a-apply-util-rs-helpers-across-command-fil"
created_at = "2026-04-12T09:02:44.386660Z"
updated_at = "2026-04-12T09:21:22.655145Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f"]
+++

## Spec

### Problem

After `apm/src/util.rs` is created (by the prerequisite ticket), the following command files still use inline boilerplate instead of the shared helpers:

- `assign.rs` — inline aggressive fetch + fetch warning + confirmation prompt
- `show.rs` — inline aggressive fetch + fetch warning
- `next.rs` — inline aggressive fetch + fetch warning
- `close.rs` — inline aggressive fetch + fetch warning
- `spec.rs` — inline aggressive fetch + fetch warning
- `sync.rs` — inline aggressive fetch + fetch warning
- `new.rs` — inline aggressive fetch

Each of these files should be updated to call `util::fetch_if_aggressive()`, `util::log_fetch_warning()`, and `util::prompt_yes_no()` instead of reimplementing the patterns inline. This is a mechanical find-and-replace across ~7 files.

Note: `clean.rs` and `epic.rs` are handled by separate tickets in this epic to avoid conflicts.

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:21Z | groomed | in_design | philippepascal |
