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

The function `gh_pr_create_or_update` in `apm-core/src/state.rs` (line ~171) passes the raw ticket title as the `--title` argument when creating a GitHub PR. The ticket ID is not included in the title, so PRs on GitHub show only a plain title with no trace back to the ticket unless you read the body.

The desired behaviour is a PR title prefixed with the short ticket ID (first 8 characters of the full UUID `id` field), e.g. `ab12cd34: Fix the thing`. The body already contains `Closes #<id>`, so the change is isolated to how the `--title` argument is constructed inside `gh_pr_create_or_update`.

### Acceptance criteria

- [ ] When `apm state <id> implemented` (or any state transition that triggers PR creation) creates a new PR, the PR title on GitHub is `<short-id>: <ticket title>` where `<short-id>` is the first 8 characters of the ticket UUID
- [ ] The PR body is unchanged and still contains `Closes #<full-id>`
- [ ] When a PR already exists for the branch, the title is not modified (existing early-return path is preserved)
- [ ] If the ticket title is empty, the PR title falls back to just the short ID prefix (no trailing colon-space)

### Out of scope

- Updating PR titles on PRs that were already created before this change
- Changing the PR body format or the `Closes #<id>` link
- Configuring the prefix format (bracket style, separator character, etc.)
- Any other call sites that construct PR titles outside of `gh_pr_create_or_update`

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:44Z | — | new | philippepascal |
| 2026-04-01T07:44Z | new | in_design | philippepascal |