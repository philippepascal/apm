+++
id = "ae4104f2"
title = "Surface config-derived recovery guidance for merge-failure states in apm CLI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ae4104f2-surface-config-derived-recovery-guidance"
created_at = "2026-05-30T02:11:03.737221Z"
updated_at = "2026-05-30T02:14:18.952503Z"
+++

## Spec

### Problem

When a ticket lands in a merge-failure state (e.g. `merge_failed` in the default workflow, though the state name is project-configurable), the supervisor has no in-context guidance on how to proceed. `apm show` prints frontmatter and history without surfacing recovery options. `apm list` filtered to the failure state prints rows with no hint. `apm next` can surface a merge-failure ticket as actionable without explaining what action to take. The supervisor must either know the conventions from memory or consult external documentation.

With config-aware surfacing, the CLI derives recovery options directly from the workflow configuration: which transition retries the merge, which returns the ticket to a worker, and which abandons it. All labels and target state IDs come from config, enforcing the order-independence discipline established by tickets ada017c0 and 27439a80 — no state name is hardcoded anywhere in the output path.

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
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:14Z | groomed | in_design | philippepascal |