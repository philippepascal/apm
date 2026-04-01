+++
id = "034ed345"
title = "apm state: include ticket ID in GitHub PR title"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "59328"
branch = "ticket/034ed345-apm-state-include-ticket-id-in-github-pr"
created_at = "2026-04-01T07:44:24.055761Z"
updated_at = "2026-04-01T07:44:46.342072Z"
+++

## Spec

### Problem

gh_pr_create_or_update in apm-core/src/state.rs (line ~186) passes the raw ticket title as the PR title. The ticket ID is not included, so PRs on GitHub show only the title with no way to trace back to the ticket without reading the body.

The PR title should be prefixed with the short ticket ID, e.g.: 'ab12cd34: Fix the thing' or '[ab12cd34] Fix the thing'. The short ID is the first 8 chars of the full UUID id field. The body already contains 'Closes #<id>' so the fix is purely in how the --title argument is constructed in gh_pr_create_or_update.

What is broken or missing, and why it matters.

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
| 2026-04-01T07:44Z | — | new | philippepascal |
| 2026-04-01T07:44Z | new | in_design | philippepascal |