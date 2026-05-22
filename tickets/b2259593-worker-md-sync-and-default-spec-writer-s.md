+++
id = "b2259593"
title = "worker_md_sync and default spec-writer sync tests fail on main — three pre-existing failures"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b2259593-worker-md-sync-and-default-spec-writer-s"
created_at = "2026-05-22T01:32:33.798879Z"
updated_at = "2026-05-22T01:59:45.597750Z"
+++

## Spec

### Problem

Three byte-for-byte sync tests in `apm-core/tests/worker_md_sync.rs` fail on main because the `apm-core/src/default/` source copies of agent markdown files have drifted from the canonical project copies in `.apm/agents/`. These failures pre-date and are unrelated to ticket 6c826abe.\n\nThe drifts are in three files:\n1. `apm-core/src/default/agents/default/apm.worker.md` and `apm-core/src/default/agents/claude/apm.worker.md`: line 6 references `apm.agents.md` while the project copies reference `.apm/agents/default/agents.md`.\n2. `apm-core/src/default/agents/default/apm.spec-writer.md` is missing content present in `.apm/agents/default/apm.spec-writer.md`: the "Never hand-edit the History table" subsection, the "Filename is fixed — never rename the ticket file" subsection, an updated step 6 in "Handling `ammend` tickets" that avoids hard-coding the ticket slug, and a corrected style.md path (`.apm/style.md` → `.apm/agents/default/style.md`).\n\nThe project copies in `.apm/agents/` are the source of truth. The `apm-core/src/default/` copies must be updated to match them.

### Acceptance criteria

- [x] `cargo test -p apm-core --test worker_md_sync default_and_project_apm_worker_md_are_identical` passes\n- [x] `cargo test -p apm-core --test worker_md_sync default_and_per_agent_apm_worker_md_are_identical` passes\n- [x] `cargo test -p apm-core --test worker_md_sync default_and_project_apm_spec_writer_md_are_identical` passes\n- [x] `cargo test --workspace` passes with no regressions

### Out of scope

- Updating any agent markdown pairs not covered by these three failing tests\n- Changes to the test files themselves\n- Changes to `.apm/agents/` project files (they are already correct)

### Approach

Three files in `apm-core/src/default/agents/` must be updated to match their `.apm/agents/` counterparts. All edits are text-only; no logic changes.\n\n**File 1 — `apm-core/src/default/agents/default/apm.worker.md`**\n\nLines 6–7: replace `Read `apm.agents.md` for startup, identity, worktree setup, and shell / discipline.` with `Read `.apm/agents/default/agents.md` for startup, identity, worktree setup, / and shell discipline.` (matching the project copy at `.apm/agents/default/apm.worker.md` line 6–7).\n\n**File 2 — `apm-core/src/default/agents/claude/apm.worker.md`**\n\nIdentical line 6–7 change as File 1.\n\n**File 3 — `apm-core/src/default/agents/default/apm.spec-writer.md`**\n\nThree edits vs the project copy at `.apm/agents/default/apm.spec-writer.md`:\n\n1. After line 50 (`Do NOT write the ticket markdown file directly.`), insert the `### Never hand-edit the History table` and `### Filename is fixed — never rename the ticket file` subsections (project copy lines 52–85).\n\n2. In the `## Handling `ammend` tickets` section, replace step 6. The source has a simple two-line git add/commit using a hardcoded `tickets/<id>-<slug>.md` path. The project copy uses `FILE=$(ls <worktree-path>/tickets/<id>-*.md)` to avoid computing the slug.\n\n3. In the `## Style rules` section at the end, change the path from `.apm/style.md` to `.apm/agents/default/style.md`.\n\nIn each case the project copy is the reference; copy it verbatim into the source file. After editing, `diff` each pair to confirm byte-for-byte identity, then run `cargo test --workspace`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T01:32Z | — | new | claude-0522-0127-3120|philippepascal |
| 2026-05-22T01:52Z | new | groomed | philippepascal |
| 2026-05-22T01:53Z | groomed | in_design | philippepascal |
| 2026-05-22T01:57Z | in_design | specd | claude-0522-0153-9a40 |
| 2026-05-22T01:59Z | specd | ready | philippepascal |
| 2026-05-22T01:59Z | ready | in_progress | philippepascal |