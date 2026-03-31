+++
id = "9d61b061"
title = "apm-server tests use live repo and hardcoded ticket ID"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0331-2000-p9x1"
branch = "ticket/9d61b061-apm-server-tests-use-live-repo-and-hardc"
created_at = "2026-03-31T21:45:15.980577Z"
updated_at = "2026-03-31T21:45:23.942013Z"
+++

## Spec

### Problem

The apm-server unit tests introduced in 54eb5bfc have two fragility issues:

1. They resolve repo_root() to the real apm repo on disk and read live ticket data from git. Tests pass only when run from a checkout that has tickets present. In CI against a clean or shallow clone this could silently produce wrong results or false positives.

2. The get_ticket_valid_prefix_returns_200_object test hardcodes the ticket ID "54eb5bfc". Once that ticket's branch is deleted (after the ticket closes), the test will return 404 and fail. The test is self-referential in a way that will break over time.

A secondary minor issue: the dev-dependency on tower 0.4.13 conflicts with axum's transitive tower 0.5.3, causing two copies of tower in Cargo.lock. Bumping to tower = "0.5" removes the duplication.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T21:45Z | — | new | claude-0331-2000-p9x1 |
| 2026-03-31T21:45Z | new | in_design | claude-0331-2000-p9x1 |