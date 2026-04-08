+++
id = "c738d9cc"
title = "Validate owner against GitHub repo collaborators"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c738d9cc-validate-owner-against-github-repo-colla"
created_at = "2026-04-08T15:10:04.160555Z"
updated_at = "2026-04-08T15:10:04.160555Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

In GitHub mode (`git_host.provider = "github"`), owner changes are not validated against the actual repo collaborators. A ticket can be assigned to a username that has no access to the repository. The existing `fetch_repo_collaborators()` in github.rs provides the mechanism but is never called at runtime.

### Acceptance criteria

- [ ] When `git_host.provider = "github"` and `git_host.repo` is set, `apm assign` validates the new owner against GitHub repo collaborators
- [ ] Uses `gh api` or token-based API call to fetch collaborators
- [ ] If the new owner is not a collaborator, command fails with a clear error
- [ ] If GitHub API is unreachable (no token, network error), validation is skipped with a warning (do not block offline work)
- [ ] Falls back to `project.collaborators` list if GitHub API fails
- [ ] Tests cover: valid GitHub collaborator accepted, unknown user rejected, API failure falls back gracefully

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
| 2026-04-08T15:10Z | — | new | philippepascal |