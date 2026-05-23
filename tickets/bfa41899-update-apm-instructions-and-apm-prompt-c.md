+++
id = "bfa41899"
title = "Update apm instructions and apm prompt CLI help for new model"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bfa41899-update-apm-instructions-and-apm-prompt-c"
created_at = "2026-05-22T23:23:41.917063Z"
updated_at = "2026-05-23T00:34:32.329502Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771", "d8e2fa0e"]
+++

## Spec

### Problem

Two CLI help strings become stale after the T1 (4bee5771) and T3 (d8e2fa0e) redesign.

`apm instructions` (line 894 of `apm/src/main.rs` and the `PREAMBLE` constant in `apm/src/cmd/instructions.rs`) currently describes the command as emitting "a compact plain-text guide" that lists commands. After T1, the command calls `apm_core::instructions::generate()` and emits full APM system knowledge across five named sections: state machine, ticket format, shell discipline, session identity, and command reference. The about string and any surviving intro text must reflect this.

`apm prompt` (the `#[command(long_about = "...")]` block in `apm/src/main.rs`, lines 842–879) documents a flat 5-level cascade (levels 0–4) and shows a `--explain` sample with `prefix:` / `system prompt:` labels. After T3, the prompt composes three layers — layer 1 (dynamic apm instructions), layer 2 (project context file), layer 3 (role-file cascade) — and `format_provenance` outputs `layer 1:` / `layer 2:` / `layer 3:` labels. The long_about and its embedded `--explain` example must describe the three-layer model and match the T3 output format.

### Acceptance criteria

- [ ] `apm instructions --help` describes the output as APM system knowledge (state machine, ticket format, shell discipline, session identity, command reference) — not as a compact command list
- [ ] `apm prompt --help` does not describe the 0–4 cascade levels as top-level composition steps
- [ ] `apm prompt --help` describes three named layers: layer 1 = apm instructions (dynamic), layer 2 = project context file, layer 3 = role-file cascade
- [ ] The `--explain` sample in `apm prompt --help` shows `layer 1:`, `layer 2:`, `layer 3:`, and `skipped:` labels, matching the T3 `format_provenance` output format
- [ ] No help text retains the old `prefix:` or `system prompt:` explain labels
- [ ] The last example line in `apm prompt --help` reads `# show layer provenance` (not `# show cascade provenance`)
- [ ] Dead-code constants and helpers in `apm/src/cmd/instructions.rs` left orphaned by T1 (`PREAMBLE`, `render`, `render_compact_commands` and their unit tests) are removed if still present after T1 lands

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
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:51Z | new | groomed | philippepascal |
| 2026-05-23T00:34Z | groomed | in_design | philippepascal |