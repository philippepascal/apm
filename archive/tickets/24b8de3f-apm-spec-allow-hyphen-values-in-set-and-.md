+++
id = "24b8de3f"
title = "apm spec: allow hyphen values in --set and add --set-file flag"
state = "closed"
priority = 90
effort = 0
risk = 0
author = "claude-0331-1200-a7b9"
agent = "11266"
branch = "ticket/24b8de3f-apm-spec-allow-hyphen-values-in-set-and-"
created_at = "2026-03-31T18:26:12.162123Z"
updated_at = "2026-04-01T04:54:35.890480Z"
+++

## Spec

### Problem

Two related usability gaps in `apm spec --set`:

1. **Clap rejects values starting with `-`**: `apm spec <id> --section "Amendment requests" --set "- [ ] Fix the thing"` fails with `error: unexpected argument '- ' found`. Clap interprets the leading dash as a flag prefix. This makes it impossible to pass checklist content (which always starts with `- [ ]`) directly on the command line.

2. **No file input option**: Multi-line content (amendment lists, full section rewrites) must currently be piped via stdin (`--set -`), which requires a heredoc or a temp file. Heredocs are forbidden by apm shell discipline; temp files require extra Write tool calls and generate file permission prompts in Claude Code's acceptEdits mode.

Both gaps force agents into awkward workarounds when writing amendment requests or multi-line spec sections.

### Acceptance criteria

- [x] `apm spec <id> --section "Amendment requests" --set "- [ ] Fix the thing"` succeeds (value starting with `-` is accepted)
- [x] `apm spec <id> --section Problem --set "- list item\n- another"` succeeds
- [x] `apm spec <id> --section Problem --set-file /path/to/content.txt` reads section content from the given file and writes it to the ticket
- [x] `--set-file` with a non-existent path returns a clear error
- [x] `--set` and `--set-file` are mutually exclusive; providing both returns an error
- [x] `--set -` (stdin) continues to work as before
- [x] `cargo test --workspace` passes

### Out of scope

- Changes to any other apm command
- Interactive editor mode (`--edit`) — not needed once `--set-file` exists
- Validation of file content format

### Approach

Both fixes are in `apm/src/cmd/spec.rs` and the clap argument definition.

**Fix 1 — allow hyphen values on `--set`**

In the clap argument definition for `--set`, add `.allow_hyphen_values(true)`. This tells clap that the value for this argument may start with a `-` and should not be interpreted as a flag.

```rust
.arg(
    Arg::new("set")
        .long("set")
        .value_name("SET")
        .allow_hyphen_values(true)  // add this
        .help("New content for the section; use \"-\" to read from stdin")
)
```

**Fix 2 — add `--set-file <PATH>`**

Add a new `--set-file` argument that reads content from a file path:

```rust
.arg(
    Arg::new("set_file")
        .long("set-file")
        .value_name("PATH")
        .conflicts_with("set")
        .help("Read new section content from this file")
)
```

In the handler, resolve the three content sources in order:
1. `--set -` → read from stdin (existing behaviour)
2. `--set <value>` → use value directly (existing behaviour, now works with leading `-`)
3. `--set-file <path>` → `std::fs::read_to_string(path)?`

Then pass the resolved content string to the existing section-write logic unchanged.

File changes:
- `apm/src/cmd/spec.rs` — add `allow_hyphen_values`, add `--set-file` arg, add file-read branch

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:26Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T18:26Z | new | in_design | claude-0331-1200-a7b9 |
| 2026-03-31T18:26Z | in_design | specd | claude-0331-1200-a7b9 |
| 2026-03-31T19:45Z | specd | ready | apm |
| 2026-03-31T20:41Z | ready | in_progress | philippepascal |
| 2026-03-31T20:47Z | in_progress | implemented | claude-0331-2041-w4k9 |
| 2026-03-31T20:51Z | implemented | accepted | apm-sync |
| 2026-04-01T04:54Z | accepted | closed | apm-sync |