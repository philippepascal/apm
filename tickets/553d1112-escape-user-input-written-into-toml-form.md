+++
id = "553d1112"
title = "escape user input written into TOML format strings"
state = "ammend"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/553d1112-escape-user-input-written-into-toml-form"
created_at = "2026-03-31T00:17:29.464358Z"
updated_at = "2026-03-31T00:21:09.656620Z"
+++

## Spec

### Problem

Several places in APM write user-supplied strings directly into raw TOML format strings using Rust `format!()` macros. If the input contains `"` or `\`, the output is invalid TOML that will fail to parse.

Known locations:
- `apm-core/src/init.rs` `default_config()`: `name` and `description` from interactive prompts
- Any other command that interpolates user input into raw TOML

All such strings must be escaped before interpolation: `\` → `\\`, `"` → `\"`.

### Acceptance criteria

- [ ] All user-supplied strings interpolated into raw TOML format strings are escaped (`\` → `\\`, `"` → `\"`) before use

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T00:17Z | — | new | apm |
| 2026-03-31T00:17Z | new | in_design | apm |
| 2026-03-31T00:20Z | in_design | specd | apm |
| 2026-03-31T00:21Z | specd | ammend | apm |
