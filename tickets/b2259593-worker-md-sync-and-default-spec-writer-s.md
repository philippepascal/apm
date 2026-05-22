+++
id = "b2259593"
title = "worker_md_sync and default spec-writer sync tests fail on main — three pre-existing failures"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b2259593-worker-md-sync-and-default-spec-writer-s"
created_at = "2026-05-22T01:32:33.798879Z"
updated_at = "2026-05-22T01:53:20.066617Z"
+++

## Spec

### Problem

Three byte-for-byte sync tests in `apm-core/tests/worker_md_sync.rs` fail on main because the `apm-core/src/default/` source copies of agent markdown files have drifted from the canonical project copies in `.apm/agents/`. These failures pre-date and are unrelated to ticket 6c826abe.\n\nThe drifts are in three files:\n1. `apm-core/src/default/agents/default/apm.worker.md` and `apm-core/src/default/agents/claude/apm.worker.md`: line 6 references `apm.agents.md` while the project copies reference `.apm/agents/default/agents.md`.\n2. `apm-core/src/default/agents/default/apm.spec-writer.md` is missing content present in `.apm/agents/default/apm.spec-writer.md`: the "Never hand-edit the History table" subsection, the "Filename is fixed — never rename the ticket file" subsection, an updated step 6 in "Handling `ammend` tickets" that avoids hard-coding the ticket slug, and a corrected style.md path (`.apm/style.md` → `.apm/agents/default/style.md`).\n\nThe project copies in `.apm/agents/` are the source of truth. The `apm-core/src/default/` copies must be updated to match them.

### Acceptance criteria

- [ ] `cargo test -p apm-core --test worker_md_sync default_and_project_apm_worker_md_are_identical` passes\n- [ ] `cargo test -p apm-core --test worker_md_sync default_and_per_agent_apm_worker_md_are_identical` passes\n- [ ] `cargo test -p apm-core --test worker_md_sync default_and_project_apm_spec_writer_md_are_identical` passes\n- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Updating any agent markdown pairs not covered by these three failing tests\n- Changes to the test files themselves\n- Changes to `.apm/agents/` project files (they are already correct)

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
| 2026-05-22T01:53Z | groomed | in_design | philippepascal |