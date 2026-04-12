+++
id = "de5c5a31"
title = "Deduplicate init.rs settings manipulation functions"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de5c5a31-deduplicate-init-rs-settings-manipulatio"
created_at = "2026-04-12T09:02:35.167384Z"
updated_at = "2026-04-12T09:08:55.488094Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

`apm/src/cmd/init.rs` (305 lines) contains two near-identical functions for manipulating `.claude/settings.json`:

- `update_claude_settings()` (lines ~158-227) — updates the project-level `.claude/settings.json`
- `update_user_claude_settings()` (lines ~230-304) — updates the user-level `~/.claude/settings.json`

Both functions perform the same operations: read JSON, navigate to `permissions.allow` array, check for existing entries, append new entries, write back. The only difference is the file path and the specific permission entries added.

This ~140 lines of duplicated logic should be a single parameterized function: `fn update_settings_json(path: &Path, entries: &[&str]) -> Result<()>`.

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:08Z | new | groomed | apm |
