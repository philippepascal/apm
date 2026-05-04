+++
id = "121a05a8"
title = "place holder: apm init is full of inconsistency"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/121a05a8-place-holder-apm-init-is-full-of-inconsi"
created_at = "2026-05-03T20:29:23.302391Z"
updated_at = "2026-05-04T02:01:31.032742Z"
+++

## Spec

### Problem

The per-agent instruction files under `.apm/agents/claude/` have accumulated inconsistencies that existing tests do not catch.

`apm init` never writes to `.apm/agents/claude/` — so any fresh project initialized from scratch would be missing both per-agent files entirely. The default templates (`apm-core/src/default/agents/claude/`) exist and are embedded in the binary via `include_str!()` in `start.rs`, but the `setup()` function in `init.rs` makes no `write_default()` calls for them.

The project's `.apm/agents/claude/apm.spec-writer.md` is missing two sections — `## Scope limits` and `## Capability limitations` — that exist in the canonical default at `apm-core/src/default/agents/claude/apm.spec-writer.md`. These sections were added to the default in a prior commit but were never propagated to the project file. As a result, spec-writer agents operating on this repo receive instructions that omit scope restrictions and the graceful-exit protocol for capability blocks. The `spec_writer_md_sync.rs` test did not catch this because it only validates the `## Style rules` section.

No sync test covers `.apm/agents/claude/apm.worker.md` at all, leaving the per-agent worker file free to diverge silently from its default.

### Acceptance criteria

- [ ] Running `apm init` on a project that has no `.apm/agents/claude/` directory creates `.apm/agents/claude/apm.worker.md` with content matching the embedded default
- [ ] Running `apm init` on a project that has no `.apm/agents/claude/` directory creates `.apm/agents/claude/apm.spec-writer.md` with content matching the embedded default
- [ ] `.apm/agents/claude/apm.spec-writer.md` in this project contains the `## Scope limits` section
- [ ] `.apm/agents/claude/apm.spec-writer.md` in this project contains the `## Capability limitations` section
- [ ] `spec_writer_md_sync.rs` fails when `.apm/agents/claude/apm.spec-writer.md` differs from the default in any section
- [ ] A new sync test fails when `.apm/agents/claude/apm.worker.md` differs from `apm-core/src/default/agents/claude/apm.worker.md`
- [ ] `cargo test --workspace` passes with all new and modified tests

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
| 2026-05-04T01:54Z | new | groomed | philippepascal |
| 2026-05-04T02:01Z | groomed | in_design | philippepascal |