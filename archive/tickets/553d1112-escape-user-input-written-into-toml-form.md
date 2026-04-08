+++
id = "553d1112"
title = "escape user input written into TOML format strings"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
agent = "95803"
branch = "ticket/553d1112-escape-user-input-written-into-toml-form"
created_at = "2026-03-31T00:17:29.464358Z"
updated_at = "2026-03-31T05:05:06.473584Z"
+++

## Spec

### Problem

Several places in APM write user-supplied strings directly into raw TOML format strings using Rust `format!()` macros. If the input contains `"` or `\`, the output is invalid TOML that will fail to parse.

Known locations:
- `apm-core/src/init.rs` `default_config()`: `name` and `description` from interactive prompts
- Any other command that interpolates user input into raw TOML

All such strings must be escaped before interpolation: `\` → `\\`, `"` → `\"`.

### Acceptance criteria

- [x] All user-supplied strings interpolated into raw TOML format strings are escaped (`\` → `\\`, `"` → `\"`) before use

### Out of scope

- Escaping values already handled by serde/toml (struct serialization via `toml::to_string`)
- TOML values not wrapped in double-quoted strings (integers, booleans, arrays)
- Validation or sanitisation of branch names beyond TOML correctness

### Approach

Add a private `fn toml_escape(s: &str) -> String` helper in `apm-core/src/init.rs` that replaces `\` with `\\` and `"` with `\"`. Apply it to every user-supplied string before interpolation in `default_config()`: `name`, `description`, `default_branch`, and `log_file` (which is derived from `name`). Add a unit test asserting that `default_config()` with a name containing `\` and `"` produces valid, parseable TOML.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T00:17Z | — | new | apm |
| 2026-03-31T00:17Z | new | in_design | apm |
| 2026-03-31T00:20Z | in_design | specd | apm |
| 2026-03-31T00:21Z | specd | ammend | apm |
| 2026-03-31T00:21Z | ammend | in_design | philippepascal |
| 2026-03-31T00:24Z | in_design | specd | claude-0331-0021-0d70 |
| 2026-03-31T00:27Z | specd | ready | apm |
| 2026-03-31T00:28Z | ready | in_progress | philippepascal |
| 2026-03-31T00:30Z | in_progress | implemented | claude-0330-2350-w4r1 |
| 2026-03-31T04:04Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |