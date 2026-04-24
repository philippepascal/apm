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

The default templates shipped by apm init (apm-core/src/default/apm.agents.md, apm.spec-writer.md, apm.worker.md) contain apm-specific references: apm-core/src/, apm/tests/integration.rs, cargo test --workspace. Users running apm init on other projects (e.g. ticker) must manually rewrite these. Expected: make the defaults generic — replace hardcoded apm-specific paths/commands with placeholders like "your project test command" or leave structure-section as "_Fill in your project structure here._" (already partially done). Reference rewrite available at /Users/philippepascal/repos/ticker/.apm/agents.md. While editing, also add a convention noting #### as an editing-subsection marker (supervisor preference used in ticker version for spec/approach edits). Related to the "supervisor-only transitions" ticket — do this one first; that one builds on top.

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
