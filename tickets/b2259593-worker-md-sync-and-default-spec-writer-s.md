+++
id = "b2259593"
title = "worker_md_sync and default spec-writer sync tests fail on main — three pre-existing failures"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b2259593-worker-md-sync-and-default-spec-writer-s"
created_at = "2026-05-22T01:32:33.798879Z"
updated_at = "2026-05-22T01:52:47.396342Z"
+++

## Spec

### Problem

Three tests in apm-core/tests/worker_md_sync.rs fail on main, unrelated to ticket 6c826abe:\n1. default_and_per_agent_apm_worker_md_are_identical: apm-core/src/default/agents/claude/apm.worker.md reads 'apm.agents.md' but .apm/agents/claude/apm.worker.md reads '.apm/agents/default/agents.md'\n2. default_and_project_apm_worker_md_are_identical: same issue for default/apm.worker.md pair\n3. default_and_project_apm_spec_writer_md_are_identical: apm-core/src/default/agents/default/apm.spec-writer.md is missing a large block (lines 52-85+) present in .apm/agents/default/apm.spec-writer.md (Never hand-edit the History table + Filename is fixed sections). Same fix pattern as 6c826abe — update the default/ source copies to match the project copies.

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
| 2026-05-22T01:32Z | — | new | claude-0522-0127-3120|philippepascal |
| 2026-05-22T01:52Z | new | groomed | philippepascal |
