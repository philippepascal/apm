+++
id = "74f9a232"
title = "apm list should have a way to format output to use in pipes"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/74f9a232-apm-list-should-have-a-way-to-format-out"
created_at = "2026-06-11T01:10:29.686451Z"
updated_at = "2026-06-11T05:35:13.178716Z"
depends_on = ["67f83715"]
+++

## Spec

### Problem

`apm list` produces a human-readable table (columns: id, state, owner, base, title) plus footer blocks for stale warnings and recovery hints. This output is hostile to pipes — extracting ticket IDs requires something like `awk '{print $1}' | sed 's/^\*//'`, which breaks whenever the stale marker or column alignment changes.

Users need a machine-readable output mode so that `apm list` results can feed directly into shell scripts, other `apm` commands, and automation pipelines. The most important use case is a flat comma-separated list of ticket IDs.

### Acceptance criteria

- [x] `apm list --format ids` prints a comma-separated list of ticket IDs on a single line (e.g. `74f9a232,3a1b2c3d`)
- [x] `apm list --format ids` with no matching tickets prints an empty line and exits 0
- [x] `apm list --format ids` respects all existing filter flags (`--state`, `--unassigned`, `--actionable`, `--mine`, `--author`, `--owner`, `--all`)
- [x] `apm list --format ids` omits the stale-ticket footer, diverged-ticket warning, and recovery hint block
- [x] `apm list --format json` prints a JSON array of objects, each containing the ticket's frontmatter fields
- [x] `apm list --format json` with no matching tickets prints `[]`
- [x] `apm list --format json` omits the stale-ticket footer, diverged-ticket warning, and recovery hint block
- [x] `apm list` without `--format` produces identical output to the current behaviour
- [x] `apm list --format <unknown>` exits non-zero with a message naming the supported values

### Out of scope

- Newline-separated output (one ID per line) — the comma format is sufficient for scripting
- Additional format values beyond `ids` and `json`
- Changes to `apm next` output format
- Changes to any other `apm` subcommand

### Approach

All changes are contained to two files: `apm/src/main.rs` and `apm/src/cmd/list.rs`. No changes to `apm-core`.

#### `apm/src/main.rs`

Add a `format` field to the `List` command variant, after the `owner` field:

```rust
/// Output format: ids (comma-separated IDs) or json (JSON array)
#[arg(long, value_name = "FORMAT")]
format: Option<String>,
```

Update the `Command::List { ... }` destructure and the `cmd::list::run(...)` call to pass `format` as the last argument.

Add `--format ids` to the `apm list` examples in the long `help` string.

#### `apm/src/cmd/list.rs`

Add `format: Option<String>` as the last parameter to `run(...)`.

After `list_filtered` returns the `filtered` vec, branch on the format value before the existing rendering loop:

- **`Some("ids")`** — Collect `fm.id` for each ticket, join with `","`, and `println!` the result (empty line when the vec is empty). Return immediately — skip all footer blocks.

- **`Some("json")`** — Build a `Vec<&Frontmatter>` from `filtered`, serialize with `serde_json::to_string(...)`, and `println!` the result (`"[]"` naturally when vec is empty). Return immediately — skip all footer blocks. `Frontmatter` already derives `serde::Serialize`, so no extra work is needed.

- **`Some(other)`** — `anyhow::bail!("unknown format {:?}; supported: ids, json", other)`

- **`None`** — fall through to the existing table rendering loop unchanged.

No new dependencies are required: `serde_json` is already in `apm/Cargo.toml`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-11T01:10Z | — | new | philippepascal |
| 2026-06-11T01:12Z | new | groomed | philippepascal |
| 2026-06-11T01:16Z | groomed | in_design | philippepascal |
| 2026-06-11T01:19Z | in_design | specd | claude |
| 2026-06-11T05:29Z | specd | ready | philippepascal |
| 2026-06-11T05:35Z | ready | in_progress | philippepascal |