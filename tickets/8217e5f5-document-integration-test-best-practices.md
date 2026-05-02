+++
id = "8217e5f5"
title = "Document integration-test best practices and bypass policy"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8217e5f5-document-integration-test-best-practices"
created_at = "2026-05-01T20:26:39.470083Z"
updated_at = "2026-05-02T03:08:24.873062Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm/tests/integration.rs has no documented convention for how tests should interact with APM. Every setup helper hand-rolls an `apm.toml`, writes ticket frontmatter directly via `std::fs::write`, and invokes raw git porcelain — with no policy on when that is acceptable. Changes to the production init template, ticket frontmatter rules, or CLI behaviour go unexercised and are invisible to the test suite.

The epic containing this ticket migrates those helpers to drive APM via real `apm` commands. That migration requires a documented standard: what is the default approach, what constitutes a legitimate bypass, and how do bypasses get flagged so they can be grepped and audited. Without this document, each migration ticket makes its own call, producing inconsistent patterns across the file.

This ticket produces the policy document. It is a prerequisite for all sibling migration tickets and establishes the principle they enforce.

### Acceptance criteria

- [ ] `apm/tests/README.md` exists and contains a "Test conventions" section
- [ ] The document states that tests must drive APM via real `apm` commands by default
- [ ] The document defines what counts as a bypass (direct `std::fs::write` on a ticket branch, hand-built frontmatter strings, raw git porcelain substituting for an `apm` command)
- [ ] The document states bypass is permitted only when no `apm` command path exists
- [ ] The document specifies the exact inline annotation format: `// BYPASS: <one-line reason>`
- [ ] The document includes the grep command to audit all bypasses (`grep -rn "BYPASS:" apm/tests/`)
- [ ] The document names `init_repo()` as the canonical starting point for new integration tests
- [ ] CLAUDE.md Tests section is updated with a one-liner pointing to `apm/tests/README.md`

### Out of scope

- Migrating any existing setup helpers to `init_repo()` — covered by sibling tickets in this epic
- Adding `// BYPASS:` annotations to existing direct-write code — each migration ticket handles its own bypasses
- Changing any Rust test code or test logic
- Enforcing the policy via CI linting or automated checks

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:08Z | groomed | in_design | philippepascal |