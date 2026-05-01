+++
id = "059e2e74"
title = "Replace direct ticket-file writes with apm new"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/059e2e74-replace-direct-ticket-file-writes-with-a"
created_at = "2026-05-01T20:27:29.576253Z"
updated_at = "2026-05-01T20:27:29.576253Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

Several tests in apm/tests/integration.rs build ticket frontmatter directly as a string (e.g. lines 999, 1113, write_ticket_to_branch usages) and write it to disk on a branch. Changes to required ticket fields, frontmatter format, or branch-naming rules don't surface in tests. Replace these direct writes with `apm new` invocations followed by `apm state` / `apm spec` calls to reach the desired state. Where the test specifically needs to inject corrupted or legacy ticket state, mark the bypass with `// BYPASS:` per the policy.

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
| 2026-05-01T20:27Z | — | new | philippepascal |
