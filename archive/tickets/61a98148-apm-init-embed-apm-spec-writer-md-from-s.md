+++
id = "61a98148"
title = "apm init: embed apm.spec-writer.md from source instead of placeholder stub"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "apm"
agent = "60691"
branch = "ticket/61a98148-apm-init-embed-apm-spec-writer-md-from-s"
created_at = "2026-04-02T02:09:54.035008Z"
updated_at = "2026-04-02T19:06:57.380663Z"
+++

## Spec

### Problem

When `apm init` sets up a new project, it creates `.apm/apm.spec-writer.md` with a minimal two-line placeholder stub. Every other template file written during init (`apm.worker.md`, `apm.agents.md`) is embedded from a real source file in `apm-core/src/` via `include_str!()`, so new projects get working, complete instructions out of the box. The spec-writer file is the only exception — it ships empty, leaving spec-writer agents with no guidance until a human manually fills it in.

This matters because spec-writer agents run autonomously on `groomed`, `ammend`, and `in_design` state tickets; they depend on `.apm/apm.spec-writer.md` for their instructions. A placeholder produces low-quality or incomplete specs.

### Acceptance criteria

- [x] After `apm init`, `.apm/apm.spec-writer.md` contains the full spec-writer instructions (more than 50 lines), not a placeholder stub
- [x] The content written to `.apm/apm.spec-writer.md` matches the content of `apm-core/src/apm.spec-writer.md` verbatim
- [x] `apm init` does not overwrite an already-existing `.apm/apm.spec-writer.md` (idempotent, same as other template files)
- [x] `cargo test --workspace` passes after the change

### Out of scope

- Changing the content of the spec-writer instructions themselves (the content is taken as-is from the current `.apm/apm.spec-writer.md` in this repo)
- Updating `apm.worker.md` or `apm.agents.md` content
- Any changes to the workflow state machine or `apm.toml` defaults
- Migration logic for projects that already have a `.apm/apm.spec-writer.md`

### Approach

Two files change:

**1. Create `/apm-core/src/apm.spec-writer.md`**
Copy the contents of the current `/.apm/apm.spec-writer.md` (the live instructions file in this repo, 147 lines) verbatim into `/apm-core/src/apm.spec-writer.md`. This is the canonical source that will be embedded at compile time.

**2. Update `/apm-core/src/init.rs` lines 47-54**
Replace the inline placeholder write:
```rust
std::fs::write(
    &spec_writer_path,
    "# APM Spec-Writer Agent\n\n_Fill in spec-writing instructions here._\n",
)?;
```
with the `include_str!` pattern already used for `apm.worker.md`:
```rust
std::fs::write(&spec_writer_path, include_str!("apm.spec-writer.md"))?;
```

No other files change. The `!spec_writer_path.exists()` guard is already in place and must be kept (no overwrite on re-init).

Order: create the source file first, then update `init.rs`, then run `cargo test --workspace`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T02:09Z | — | new | apm |
| 2026-04-02T02:10Z | new | groomed | apm |
| 2026-04-02T02:12Z | groomed | in_design | philippepascal |
| 2026-04-02T02:14Z | in_design | specd | claude-0401-0000-sw01 |
| 2026-04-02T02:29Z | specd | ready | apm |
| 2026-04-02T07:04Z | ready | in_progress | philippepascal |
| 2026-04-02T07:06Z | in_progress | implemented | claude-0402-0704-w61a |
| 2026-04-02T19:06Z | implemented | closed | apm-sync |