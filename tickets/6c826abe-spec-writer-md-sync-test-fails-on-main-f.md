+++
id = "6c826abe"
title = "spec_writer_md_sync test fails on main — files diverged"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6c826abe-spec-writer-md-sync-test-fails-on-main-f"
created_at = "2026-05-21T23:42:35.079918Z"
updated_at = "2026-05-22T01:25:24.298290Z"
+++

## Spec

### Problem

The test apm-core/tests/spec_writer_md_sync.rs::default_and_per_agent_apm_spec_writer_md_are_identical fails on the main branch (not introduced by ticket ba121f45). It detects that apm-core/src/default/agents/claude/apm.spec-writer.md and .apm/agents/claude/apm.spec-writer.md have diverged at line 236: the default file references .apm/style.md while the project file references .apm/agents/default/style.md. These two files need to be kept in sync.

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
| 2026-05-21T23:42Z | — | new | claude-0521-2330-ba12|philippepascal |
| 2026-05-22T01:25Z | new | groomed | philippepascal |
