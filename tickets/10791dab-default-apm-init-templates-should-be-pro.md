+++
id = "10791dab"
title = "Default apm init templates should be project-agnostic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/10791dab-default-apm-init-templates-should-be-pro"
created_at = "2026-04-24T06:28:34.301755Z"
updated_at = "2026-04-24T07:14:39.318393Z"
+++

## Spec

### Problem

The three default templates shipped by `apm init` — `apm.agents.md`, `apm.spec-writer.md`, and `apm.worker.md` — contain hardcoded references to the APM project's own codebase. Specifically:

- `apm.worker.md` names `apm-core/src/` and `apm-core/tests/` as the locations for unit tests, and `apm/tests/integration.rs` as the integration test file. It also hard-codes `cargo test --workspace` as the test command.
- `apm.agents.md` hard-codes `cargo test --workspace` in both the Development workflow list and the shell-discipline section's `bash -c` example.

When a user runs `apm init` in a new project (e.g. a Python service, a Go CLI, or the `ticker` repo), these files land verbatim in `.apm/`. The agent that reads them gets wrong path references and a wrong test command. The user must manually rewrite three files every time.

The desired behaviour: the defaults should be project-agnostic placeholders. Cargo- and APM-path-specific text should be replaced with phrasing like "Run your project's test suite" and "Write tests appropriate for your project's structure." The `## Repo structure` section of `apm.agents.md` is already generic (`_Fill in your project's structure here._`) and is the model for the rest.

A second gap: the templates do not document the `####` subsection convention. Supervisors and spec-writers use `####` headings inside long sections (e.g. `### Approach`) as editing handles — targeted `apm spec --section` calls can update a named subsection without rewriting the whole section. This convention exists in the ticker fork but is absent from the defaults.

Affected users: any developer who runs `apm init` on a non-APM project — the primary use case for `apm init`. The friction is immediate and requires manual cleanup of three files.

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
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |