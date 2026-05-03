+++
id = "121a05a8"
title = "place holder: apm init is full of inconsistency"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/121a05a8-place-holder-apm-init-is-full-of-inconsi"
created_at = "2026-05-03T20:29:23.302391Z"
updated_at = "2026-05-03T20:29:23.302391Z"
+++

## Spec

### Problem

strategy for agents md file is unclear. need generic linking to claude
 The cleanup pile when you're ready:
  - apm init is unaware of per-agent files
  - .apm/agents/claude/apm.worker.md doesn't exist
  - .apm/agents/claude/apm.spec-writer.md is missing the two sections added by 9fcc94ed
  - No sync tests cover the per-agent apm.worker.md at all
  - The spec_writer_md_sync test only compares the ## Style rules section, not the full file

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
| 2026-05-03T20:29Z | — | new | philippepascal |