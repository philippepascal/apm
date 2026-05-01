+++
id = "8217e5f5"
title = "Document integration-test best practices and bypass policy"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8217e5f5-document-integration-test-best-practices"
created_at = "2026-05-01T20:26:39.470083Z"
updated_at = "2026-05-01T20:26:39.470083Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

Tests in apm/tests/integration.rs hand-roll their own apm.toml, write ticket frontmatter directly, and bypass apm CLI commands ad hoc, with no policy on when bypass is acceptable. Add a short doc (apm/tests/README.md or similar) stating: tests must drive APM via real `apm` commands by default; bypass (raw filesystem ops, hand-built frontmatter, direct git porcelain on ticket branches) is only permitted when no command path exists, and every bypass must be flagged inline with `// BYPASS: <one-line reason>` so we can grep-audit them. Foundational ticket — sets the principle the migration tickets enforce.

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
| 2026-05-01T20:26Z | — | new | philippepascal |
