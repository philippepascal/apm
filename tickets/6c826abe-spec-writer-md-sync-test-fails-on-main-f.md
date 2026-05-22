+++
id = "6c826abe"
title = "spec_writer_md_sync test fails on main — files diverged"
state = "specd"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6c826abe-spec-writer-md-sync-test-fails-on-main-f"
created_at = "2026-05-21T23:42:35.079918Z"
updated_at = "2026-05-22T01:27:09.031755Z"
+++

## Spec

### Problem

The test apm-core/tests/spec_writer_md_sync.rs::default_and_per_agent_apm_spec_writer_md_are_identical fails on the main branch (not introduced by ticket ba121f45). It detects that apm-core/src/default/agents/claude/apm.spec-writer.md and .apm/agents/claude/apm.spec-writer.md have diverged at line 236: the default file references .apm/style.md while the project file references .apm/agents/default/style.md. These two files need to be kept in sync.

### Acceptance criteria

- [ ] `cargo test -p apm-core spec_writer_md_sync` passes on main\n- [ ] `diff apm-core/src/default/agents/claude/apm.spec-writer.md .apm/agents/claude/apm.spec-writer.md` produces no output

### Out of scope

- Changing the referenced style path to anything other than the value already in the project file\n- Updating any other agent instruction files\n- Investigating why the two files diverged or adding a CI gate to prevent future drift

### Approach

The divergence is a single line in `apm-core/src/default/agents/claude/apm.spec-writer.md`. The project copy (`.apm/agents/claude/apm.spec-writer.md`) was updated to reference `.apm/agents/default/style.md`, but the default copy was not updated in step.\n\n**File to change:** `apm-core/src/default/agents/claude/apm.spec-writer.md`\n\n**Change:** Line 236 — in the Style rules paragraph, replace the old style-file path:\n\n- old: `read \`.apm/style.md\` if present`\n- new: `read \`.apm/agents/default/style.md\` if present`\n\nAfter the edit, verify with `diff apm-core/src/default/agents/claude/apm.spec-writer.md .apm/agents/claude/apm.spec-writer.md` (expect no output), then confirm with `cargo test -p apm-core spec_writer_md_sync`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-21T23:42Z | — | new | claude-0521-2330-ba12|philippepascal |
| 2026-05-22T01:25Z | new | groomed | philippepascal |
| 2026-05-22T01:25Z | groomed | in_design | philippepascal |
| 2026-05-22T01:27Z | in_design | specd | claude-0522-0125-f408 |
