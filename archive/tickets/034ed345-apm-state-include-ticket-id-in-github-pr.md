+++
id = "034ed345"
title = "apm state: include ticket ID in GitHub PR title"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "94174"
branch = "ticket/034ed345-apm-state-include-ticket-id-in-github-pr"
created_at = "2026-04-01T07:44:24.055761Z"
updated_at = "2026-04-01T21:29:04.898003Z"
+++

## Spec

### Problem

The function `gh_pr_create_or_update` in `apm-core/src/state.rs` (line ~171) passes the raw ticket title as the `--title` argument when creating a GitHub PR. The ticket ID is not included in the title, so PRs on GitHub show only a plain title with no trace back to the ticket unless you read the body.

The desired behaviour is a PR title prefixed with the short ticket ID (first 8 characters of the full UUID `id` field), e.g. `ab12cd34: Fix the thing`. The body already contains `Closes #<id>`, so the change is isolated to how the `--title` argument is constructed inside `gh_pr_create_or_update`.

### Acceptance criteria

- [x] When `apm state <id> implemented` (or any state transition that triggers PR creation) creates a new PR, the PR title on GitHub is `<short-id>: <ticket title>` where `<short-id>` is the first 8 characters of the ticket UUID
- [x] The PR body is unchanged and still contains `Closes #<full-id>`
- [x] When a PR already exists for the branch, the title is not modified (existing early-return path is preserved)
- [x] If the ticket title is empty, the PR title falls back to just the short ID prefix (no trailing colon-space)

### Out of scope

- Updating PR titles on PRs that were already created before this change
- Changing the PR body format or the `Closes #<id>` link
- Configuring the prefix format (bracket style, separator character, etc.)
- Any other call sites that construct PR titles outside of `gh_pr_create_or_update`

### Approach

**File:** `apm-core/src/state.rs`

**Change:** Inside `gh_pr_create_or_update`, construct the PR title by prepending the short ID before building the `--title` argument:

```rust
// line ~183, before the gh pr create call
let short_id = &id[..8.min(id.len())];
let pr_title = if title.is_empty() {
    short_id.to_string()
} else {
    format!("{short_id}: {title}")
};
```

Then replace `title` with `&pr_title` in the `.args([... "--title", title, ...])` call on line ~186.

**No other files need to change.** The `id` parameter is already passed into the function alongside `title`, so no signature change is needed.

**Test:** Add a unit or integration test that calls the state transition that triggers PR creation and asserts the PR title starts with the 8-char short ID. Because `gh` is a live CLI, the test can be a pure string-construction test on the formatting logic extracted into a small helper, or an integration test that stubs the command.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T07:44Z | — | new | philippepascal |
| 2026-04-01T07:44Z | new | in_design | philippepascal |
| 2026-04-01T07:45Z | in_design | specd | claude-0401-0744-6ee0 |
| 2026-04-01T07:56Z | specd | ready | apm |
| 2026-04-01T07:57Z | ready | in_progress | philippepascal |
| 2026-04-01T08:00Z | in_progress | implemented | claude-0401-0757-66f0 |
| 2026-04-01T08:03Z | implemented | accepted | apm |
| 2026-04-01T21:29Z | accepted | closed | apm-sync |