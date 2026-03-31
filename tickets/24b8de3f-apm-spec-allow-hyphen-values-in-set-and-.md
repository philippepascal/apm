+++
id = "24b8de3f"
title = "apm spec: allow hyphen values in --set and add --set-file flag"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0331-1200-a7b9"
branch = "ticket/24b8de3f-apm-spec-allow-hyphen-values-in-set-and-"
created_at = "2026-03-31T18:26:12.162123Z"
updated_at = "2026-03-31T18:26:20.477438Z"
+++

## Spec

### Problem

Two related usability gaps in `apm spec --set`:

1. **Clap rejects values starting with `-`**: `apm spec <id> --section "Amendment requests" --set "- [ ] Fix the thing"` fails with `error: unexpected argument '- ' found`. Clap interprets the leading dash as a flag prefix. This makes it impossible to pass checklist content (which always starts with `- [ ]`) directly on the command line.

2. **No file input option**: Multi-line content (amendment lists, full section rewrites) must currently be piped via stdin (`--set -`), which requires a heredoc or a temp file. Heredocs are forbidden by apm shell discipline; temp files require extra Write tool calls and generate file permission prompts in Claude Code's acceptEdits mode.

Both gaps force agents into awkward workarounds when writing amendment requests or multi-line spec sections.

### Acceptance criteria

- [ ] `apm spec <id> --section "Amendment requests" --set "- [ ] Fix the thing"` succeeds (value starting with `-` is accepted)
- [ ] `apm spec <id> --section Problem --set "- list item\n- another"` succeeds
- [ ] `apm spec <id> --section Problem --set-file /path/to/content.txt` reads section content from the given file and writes it to the ticket
- [ ] `--set-file` with a non-existent path returns a clear error
- [ ] `--set` and `--set-file` are mutually exclusive; providing both returns an error
- [ ] `--set -` (stdin) continues to work as before
- [ ] `cargo test --workspace` passes

### Out of scope

- Changes to any other apm command
- Interactive editor mode (`--edit`) — not needed once `--set-file` exists
- Validation of file content format

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:26Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T18:26Z | new | in_design | claude-0331-1200-a7b9 |