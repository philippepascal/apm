+++
id = "9d61b061"
title = "apm-server tests use live repo and hardcoded ticket ID"
state = "in_progress"
priority = 75
effort = 2
risk = 1
author = "claude-0331-2000-p9x1"
agent = "19386"
branch = "ticket/9d61b061-apm-server-tests-use-live-repo-and-hardc"
created_at = "2026-03-31T21:45:15.980577Z"
updated_at = "2026-03-31T23:13:32.074888Z"
+++

## Spec

### Problem

The apm-server unit tests introduced in 54eb5bfc have two fragility issues:

1. They resolve repo_root() to the real apm repo on disk and read live ticket data from git. Tests pass only when run from a checkout that has tickets present. In CI against a clean or shallow clone this could silently produce wrong results or false positives.

2. The get_ticket_valid_prefix_returns_200_object test hardcodes the ticket ID "54eb5bfc". Once that ticket's branch is deleted (after the ticket closes), the test will return 404 and fail. The test is self-referential in a way that will break over time.

A secondary minor issue: the dev-dependency on tower 0.4.13 conflicts with axum's transitive tower 0.5.3, causing two copies of tower in Cargo.lock. Bumping to tower = "0.5" removes the duplication.

### Acceptance criteria

- [ ] list_tickets and get_ticket tests use an isolated in-process ticket store, not the live repo on disk
- [ ] No test hardcodes a real ticket ID from the apm repo
- [ ] tower dev-dependency is on 0.5, eliminating the duplicate in Cargo.lock
- [ ] All four existing test cases (200 list, 200 detail, 404, 400) continue to pass

### Out of scope

Adding new test cases beyond the four existing ones; changes to the handler logic itself; integration tests that spin up a real TCP listener.

### Approach

1. Add a helper in the test module that builds an AppState from a hardcoded list of Ticket structs rather than calling load_all_from_git. This requires exposing or stubbing the state construction — either by making AppState fields pub(crate) and constructing it directly in tests, or by adding a build_app_with_tickets(tickets: Vec<Ticket>) variant of build_app for test use.

2. Construct two or three synthetic Ticket values inline in the test (fake IDs like "aaaabbbb" and "ccccdddd", minimal frontmatter fields) and verify the endpoints return the expected shapes.

3. Replace the get_ticket_valid_prefix_returns_200_object test to use one of the synthetic ticket IDs instead of "54eb5bfc".

4. In apm-server/Cargo.toml, bump tower dev-dependency from "0.4" to "0.5".

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T21:45Z | — | new | claude-0331-2000-p9x1 |
| 2026-03-31T21:45Z | new | in_design | claude-0331-2000-p9x1 |
| 2026-03-31T21:46Z | in_design | specd | claude-0331-2000-p9x1 |
| 2026-03-31T21:46Z | specd | ready | claude-0331-2000-p9x1 |
| 2026-03-31T23:13Z | ready | in_progress | philippepascal |