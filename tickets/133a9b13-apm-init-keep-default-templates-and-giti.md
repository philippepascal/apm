+++
id = "133a9b13"
title = "apm init: keep default templates and gitignore entries in sync with new features"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/133a9b13-apm-init-keep-default-templates-and-giti"
created_at = "2026-04-03T23:40:56.352188Z"
updated_at = "2026-04-04T06:31:32.857559Z"
+++

## Spec

### Problem

`apm init` installs default templates (workflow.toml, ticket.toml, config.toml, agents.md, apm.worker.md, apm.spec-writer.md) and maintains `.gitignore` entries. These defaults drift out of sync as new features land:

- The `default_workflow_toml()` in `init.rs` was missing `dep_requires` and `satisfies_deps` tags after the phase-aware dependency gating feature (b8e9bfee) landed — fixed ad-hoc but should have been part of that ticket.
- The gitignore entries list needs `.apm/sessions.json` and `.apm/credentials.json` once the auth infrastructure (e2e3d958, 8a08637c) lands.
- The `write_default` comparison mechanism (skip/replace/compare) was added to help users detect drift, but the default templates themselves must stay current for the comparison to be useful.

This ticket is a reminder to audit `default_workflow_toml()`, `default_config()`, `default_ticket_toml()`, `ensure_gitignore()`, and the `include_str!` templates in `init.rs` after each epic milestone, and to add a test that parses each default template with `Config::load` to catch structural regressions.

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
| 2026-04-03T23:40Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:31Z | groomed | in_design | philippepascal |
