+++
id = "c36a4bf6"
title = "Move embedded assets to src/default/"
state = "implemented"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c36a4bf6-move-embedded-assets-to-src-default"
created_at = "2026-04-12T06:04:13.294338Z"
updated_at = "2026-04-12T07:15:13.772703Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

The `apm-core/src/` directory mixes Rust source files with five embedded template/config assets: `apm.agents.md`, `apm.spec-writer.md`, `apm.worker.md`, `ticket.toml`, and `workflow.toml`. These files are compiled into the binary via `include_str!()` in `init.rs` and written to the user's `.apm/` directory during `apm init`. Because they live at the same level as the `.rs` modules, scanning the source tree for code files requires mentally filtering out non-code assets.

Moving these assets to `apm-core/src/default/` groups all embedded defaults in one place, making the source layout self-documenting: `src/*.rs` is code, `src/default/` is data.

### Acceptance criteria

- [x] `apm-core/src/default/apm.agents.md` exists with identical content to the pre-move `apm-core/src/apm.agents.md`
- [x] `apm-core/src/default/apm.spec-writer.md` exists with identical content to the pre-move `apm-core/src/apm.spec-writer.md`
- [x] `apm-core/src/default/apm.worker.md` exists with identical content to the pre-move `apm-core/src/apm.worker.md`
- [x] `apm-core/src/default/ticket.toml` exists with identical content to the pre-move `apm-core/src/ticket.toml`
- [x] `apm-core/src/default/workflow.toml` exists with identical content to the pre-move `apm-core/src/workflow.toml`
- [x] None of the five asset files remain at `apm-core/src/` (top level)
- [x] `cargo build -p apm-core` succeeds after the move
- [x] `cargo test -p apm-core` passes with no regressions

### Out of scope

- Changing the content of any asset file
- Adding new default assets
- Other sections of any broader refactor plan (module restructuring, etc.)
- Moving or changing `Cargo.toml` or build configuration

### Approach

All changes are confined to `apm-core/`.

1. Create `apm-core/src/default/` and move the five asset files using `git mv` (preserves history):
   - `apm-core/src/apm.agents.md` → `apm-core/src/default/apm.agents.md`
   - `apm-core/src/apm.spec-writer.md` → `apm-core/src/default/apm.spec-writer.md`
   - `apm-core/src/apm.worker.md` → `apm-core/src/default/apm.worker.md`
   - `apm-core/src/ticket.toml` → `apm-core/src/default/ticket.toml`
   - `apm-core/src/workflow.toml` → `apm-core/src/default/workflow.toml`

2. Update all five `include_str!()` calls in `apm-core/src/init.rs` (grep for `include_str!` to locate them):
   - `include_str!("apm.spec-writer.md")` → `include_str!("default/apm.spec-writer.md")`
   - `include_str!("apm.worker.md")` → `include_str!("default/apm.worker.md")`
   - `include_str!("apm.agents.md")` → `include_str!("default/apm.agents.md")`
   - `include_str!("workflow.toml")` → `include_str!("default/workflow.toml")`
   - `include_str!("ticket.toml")` → `include_str!("default/ticket.toml")`

3. Verify with `cargo test -p apm-core`. The compiler enforces correctness at build time — any missed path is a compile error. The existing test suite (`setup_creates_expected_files`, `default_workflow_toml_is_valid`, `default_ticket_toml_is_valid`, etc.) validates that the embedded content remains correct.

No other files reference these assets by source path. The `include_str!()` paths are relative to `init.rs`, so prefixing each with `default/` is the complete change.

### Open questions


### Amendment requests

- [x] Fix `\n` formatting throughout all sections — literal backslash-n characters appear instead of real newlines in Problem, Acceptance criteria, Out of scope, and Approach. Rewrite all sections with actual newlines.
- [x] Remove specific line number references (123, 124, 234, 313, 317) from the Approach section. Replace with "all `include_str!()` calls in `init.rs`" — there are exactly 5 and they're easy to grep for. Line numbers drift as other tickets land.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:11Z | new | groomed | apm |
| 2026-04-12T06:12Z | groomed | in_design | philippepascal |
| 2026-04-12T06:14Z | in_design | specd | claude-0412-0612-eb58 |
| 2026-04-12T06:53Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T06:57Z | ammend | in_design | philippepascal |
| 2026-04-12T06:58Z | in_design | specd | claude-0412-0657-7698 |
| 2026-04-12T07:12Z | specd | ready | apm |
| 2026-04-12T07:13Z | ready | in_progress | philippepascal |
| 2026-04-12T07:15Z | in_progress | implemented | claude-0412-0713-e350 |
