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
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:08Z | groomed | in_design | philippepascal |