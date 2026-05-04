+++
id = "121a05a8"
title = "place holder: apm init is full of inconsistency"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/121a05a8-place-holder-apm-init-is-full-of-inconsi"
created_at = "2026-05-03T20:29:23.302391Z"
updated_at = "2026-05-04T02:37:11.803003Z"
+++

## Spec

### Problem

The per-agent instruction files under `.apm/agents/claude/` have accumulated inconsistencies that existing tests do not catch.

`apm init` never writes to `.apm/agents/claude/` — so any fresh project initialized from scratch would be missing both per-agent files entirely. The default templates (`apm-core/src/default/agents/claude/`) exist and are embedded in the binary via `include_str!()` in `start.rs`, but the `setup()` function in `init.rs` makes no `write_default()` calls for them.

The project's `.apm/agents/claude/apm.spec-writer.md` is missing two sections — `## Scope limits` and `## Capability limitations` — that exist in the canonical default at `apm-core/src/default/agents/claude/apm.spec-writer.md`. These sections were added to the default in a prior commit but were never propagated to the project file. As a result, spec-writer agents operating on this repo receive instructions that omit scope restrictions and the graceful-exit protocol for capability blocks. The `spec_writer_md_sync.rs` test did not catch this because it only validates the `## Style rules` section.

No sync test covers `.apm/agents/claude/apm.worker.md` at all, leaving the per-agent worker file free to diverge silently from its default.

### Acceptance criteria

- [x] `.apm/agents/claude/apm.spec-writer.md` in this project contains the `## Scope limits` section
- [x] `.apm/agents/claude/apm.spec-writer.md` in this project contains the `## Capability limitations` section
- [ ] `spec_writer_md_sync.rs` fails when `.apm/agents/claude/apm.spec-writer.md` differs from the default in any section
- [ ] A new sync test fails when `.apm/agents/claude/apm.worker.md` differs from `apm-core/src/default/agents/claude/apm.worker.md`
- [ ] `cargo test --workspace` passes with all new and modified tests

### Out of scope

- Content changes to the default agent instruction files (only the project file and tests change)
- Per-agent files for non-Claude agents (debug, mock-happy, mock-random, mock-sad)
- Top-level `.apm/apm.spec-writer.md` and `.apm/apm.worker.md` (already byte-for-byte tested by `worker_md_sync.rs`)
- Changes to `config.toml` or how worker profiles reference instruction files
- Adding `apm init` integration tests (unit-level sync tests are sufficient)

### Approach

#### Step 1: Fix `.apm/agents/claude/apm.spec-writer.md`

Copy the `## Scope limits` block (lines 9–32 in the default) and the `## Capability limitations` block (lines 206–227 in the default) from `apm-core/src/default/agents/claude/apm.spec-writer.md` into the project's `.apm/agents/claude/apm.spec-writer.md`. Insert them at the correct positions so the file is byte-for-byte identical to the default. Verify with `diff` before moving on.

#### Step 2: Upgrade `spec_writer_md_sync.rs` to full-file comparison

Replace the section-extraction approach with byte-for-byte comparison, matching the pattern already used in `worker_md_sync.rs`. Specifically:
- Remove the `extract_style_rules_section()` helper
- Rename the test to `default_and_per_agent_apm_spec_writer_md_are_identical`
- Update the diff logic to produce line-level output on mismatch (copy the pattern from `worker_md_sync.rs`)
- Update the doc comment to reflect that the full file must match

The comparison paths are:
- Default: `apm-core/src/default/agents/claude/apm.spec-writer.md`
- Project: `.apm/agents/claude/apm.spec-writer.md`

#### Step 3: Add per-agent worker sync test

In `apm-core/tests/worker_md_sync.rs`, add a new test `default_and_per_agent_apm_worker_md_are_identical` that compares:
- Default: `apm-core/src/default/agents/claude/apm.worker.md` (path relative to `CARGO_MANIFEST_DIR`)
- Project: `.apm/agents/claude/apm.worker.md` (path via `CARGO_MANIFEST_DIR/../`)

Use the same byte-for-byte diff-on-failure pattern as the existing two tests in that file.

#### Step 4: Update `apm-core/src/init.rs` to write per-agent files

In `setup()`, after the existing `write_default()` call for `.apm/apm.worker.md` (currently line 134), add directory creation and two new `write_default()` calls:

```rust
let agents_claude_dir = apm_dir.join("agents/claude");
std::fs::create_dir_all(&agents_claude_dir)
    .map_err(|e| anyhow::anyhow!("cannot create {}: {e}", agents_claude_dir.display()))?;
write_default(
    &agents_claude_dir.join("apm.spec-writer.md"),
    include_str!("default/agents/claude/apm.spec-writer.md"),
    ".apm/agents/claude/apm.spec-writer.md",
    &mut messages,
)?;
write_default(
    &agents_claude_dir.join("apm.worker.md"),
    include_str!("default/agents/claude/apm.worker.md"),
    ".apm/agents/claude/apm.worker.md",
    &mut messages,
)?;
```

The `include_str!()` paths are relative to `apm-core/src/`, matching the existing pattern. The `write_default()` signature is `(path: &Path, content: &str, display_name: &str, messages: &mut Vec<InitMessage>) -> Result<()>`.

#### Order matters

Do Steps 1 and 4 before running tests — Step 1 fixes the project file so the Step 2 test passes; Step 4 changes only runtime behavior, not test assertions.
**Step 2b: Copy the default worker file into the project**

Copy `apm-core/src/default/agents/claude/apm.worker.md` to `.apm/agents/claude/apm.worker.md` in the repo. Without this file present and matching, the sync test added in Step 3 fails on a clean checkout. Verify with `diff apm-core/src/default/agents/claude/apm.worker.md .apm/agents/claude/apm.worker.md` before moving on. This step belongs between Step 2 and Step 3; the updated ordering in "Order matters" below supersedes the original.
**Order matters (updated)**

Do Steps 1, 2, and 2b before running tests — Step 1 fixes the spec-writer project file so the Step 2 test passes; Step 2b creates the worker project file so the Step 3 test passes. Step 4 changes only runtime behavior, not test assertions, and can be done at any point. The original "Order matters" block above should be considered superseded by this one.

### Open questions


### Amendment requests

- [x] Add a step to create the missing project file: copy `apm-core/src/default/agents/claude/apm.worker.md` to `.apm/agents/claude/apm.worker.md` in the repo. Without this, the test added in Step 3 fails immediately on a clean checkout. This should be an explicit step (e.g. Step 2b) in the Approach, before the test is added.
- [x] Resolve the contradiction between ACs #1–#2 and Out of Scope. Option A: remove 'Adding `apm init` integration tests' from Out of Scope and add a unit-level test that calls `init::setup()` directly in a temp dir, verifying the per-agent files are created. Option B: remove ACs #1 and #2, since the sync tests (ACs #5–#6) enforce file correctness and the init.rs code change is verified by code review. Pick one and update both the AC list and Out of Scope to be consistent.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-03T20:29Z | — | new | philippepascal |
| 2026-05-04T01:54Z | new | groomed | philippepascal |
| 2026-05-04T02:01Z | groomed | in_design | philippepascal |
| 2026-05-04T02:07Z | in_design | specd | claude-0504-0201-d860 |
| 2026-05-04T02:14Z | specd | ammend | philippepascal |
| 2026-05-04T02:20Z | ammend | in_design | philippepascal |
| 2026-05-04T02:25Z | in_design | specd | claude-0504-0220-71c0 |
| 2026-05-04T02:36Z | specd | ready | philippepascal |
| 2026-05-04T02:37Z | ready | in_progress | philippepascal |