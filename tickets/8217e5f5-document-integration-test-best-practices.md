+++
id = "8217e5f5"
title = "Document integration-test best practices and bypass policy"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8217e5f5-document-integration-test-best-practices"
created_at = "2026-05-01T20:26:39.470083Z"
updated_at = "2026-05-03T20:49:28.297095Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm/tests/integration.rs has no documented convention for how tests should interact with APM. Every setup helper hand-rolls an `apm.toml`, writes ticket frontmatter directly via `std::fs::write`, and invokes raw git porcelain — with no policy on when that is acceptable. Changes to the production init template, ticket frontmatter rules, or CLI behaviour go unexercised and are invisible to the test suite.

The epic containing this ticket migrates those helpers to drive APM via real `apm` commands. That migration requires a documented standard: what is the default approach, what constitutes a legitimate bypass, and how do bypasses get flagged so they can be grepped and audited. Without this document, each migration ticket makes its own call, producing inconsistent patterns across the file.

This ticket produces the policy document. It is a prerequisite for all sibling migration tickets and establishes the principle they enforce.

### Acceptance criteria

- [x] `apm/tests/README.md` exists and contains a "Test conventions" section
- [x] The document states that tests must drive APM via real `apm` commands by default
- [x] The document defines what counts as a bypass (direct `std::fs::write` on a ticket branch, hand-built frontmatter strings, raw git porcelain substituting for an `apm` command)
- [ ] The document states bypass is permitted only when no `apm` command path exists
- [ ] The document specifies the exact inline annotation format: `// BYPASS: <one-line reason>`
- [ ] The document includes the grep command to audit all bypasses (`grep -rn "BYPASS:" apm/tests/`)
- [ ] The document names `init_repo()` as the canonical starting point for new integration tests
- [ ] `.apm/agents.md` Tests section is updated with a one-liner pointing to `apm/tests/README.md`

### Out of scope

- Migrating any existing setup helpers to `init_repo()` — covered by sibling tickets in this epic
- Adding `// BYPASS:` annotations to existing direct-write code — each migration ticket handles its own bypasses
- Changing any Rust test code or test logic
- Enforcing the policy via CI linting or automated checks

### Approach

1. Create `apm/tests/README.md` with a "Test conventions" section covering:

   **Default rule:** Tests exercise APM through real `apm` CLI invocations (via the test harness helpers that call `Command::new("apm")` or equivalent). Tests must not substitute raw filesystem writes, hand-built frontmatter strings, or direct git porcelain for commands that `apm` exposes.

   **Bypass definition:** A bypass is any of:
   - `std::fs::write` (or equivalent) that creates or modifies a ticket file on a branch
   - A hand-constructed frontmatter string passed to git directly
   - A raw git command acting on the ticket namespace in place of an `apm` command

   **When bypass is permitted:** Only when no `apm` command path exists — for example, seeding branches into a bare origin repo, injecting intentionally corrupt state to test error paths, or overriding a config field that has no setter command.

   **Required annotation:** Every bypass line must carry an inline comment immediately above or on the same line:
   ```
   // BYPASS: <one-line reason why no apm command covers this>
   ```

   **Grep audit:** `grep -rn "BYPASS:" apm/tests/`

   **Starting point for new tests:** Use the `init_repo()` helper (added by ticket 795dce11) rather than hand-rolling setup. Compose targeted overrides on top of it.

2. Update the `## Tests` section in `.apm/agents.md` to append one sentence: "See `apm/tests/README.md` for integration-test conventions and the bypass policy."

No Rust code changes. No existing tests are modified.

### Open questions

**Q:** - Blocked: Edit tool denied permission to write CLAUDE.md (path outside the worktree). CLAUDE.md is in gitignore as intentional AI-agent-local config and cannot be committed. The acceptance criterion requiring the CLAUDE.md Tests section update needs supervisor resolution: either grant write permission for the gitignored CLAUDE.md outside the worktree, or waive the CLAUDE.md criterion given it is not committable.
A: CLAUDE.md is the wrong target. Acceptance Criteria has been updated.

### Amendment requests


### Code review
## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:08Z | groomed | in_design | philippepascal |
| 2026-05-02T03:12Z | in_design | specd | claude-0502-0308-d680 |
| 2026-05-03T20:16Z | specd | ready | philippepascal |
| 2026-05-03T20:30Z | ready | in_progress | philippepascal |
| 2026-05-03T20:34Z | in_progress | blocked | claude-0503-2030-99b8 |
| 2026-05-03T20:42Z | blocked | ready | philippepascal |
| 2026-05-03T20:49Z | ready | in_progress | philippepascal |