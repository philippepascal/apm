+++
id = "941e57fa"
title = "Make pr_or_epic_merge the default strategy and document tradeoffs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/941e57fa-make-pr-or-epic-merge-the-default-strate"
created_at = "2026-04-27T20:27:54.114826Z"
updated_at = "2026-04-27T20:44:02.372454Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

The `in_progress → implemented` transition currently uses `completion = "merge"` in `.apm/workflow.toml:152` (with `pr_or_epic_merge` commented out on line 151). The spec at `docs/strategy-and-dependencies.md` makes `pr_or_epic_merge` the recommended and default strategy because it implements the two-tier model (PR-on-main for parallel supervised work, direct-merge-to-epic for autonomous serial work) with a single setting.

Switch the default to `pr_or_epic_merge` and update user-facing documentation (`docs/agents.md`, README sections covering merging) to document the tradeoffs of each completion strategy as captured in the spec table:
- pr_or_epic_merge: composes deps within an epic (recommended)
- merge: composes deps when ticket and deps share target_branch
- pr: state→implemented fires before merge — deps unsafe
- none: nothing lands automatically — deps unsafe

See docs/strategy-and-dependencies.md sections 'Recommended default' and 'Dependency rules per strategy'.

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
| 2026-04-27T20:27Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:44Z | groomed | in_design | philippepascal |
