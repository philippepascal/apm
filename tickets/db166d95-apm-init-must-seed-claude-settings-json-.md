+++
id = "db166d95"
title = "apm init must seed .claude/settings.json with worker-essential allow-list"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db166d95-apm-init-must-seed-claude-settings-json-"
created_at = "2026-05-15T01:21:17.353568Z"
updated_at = "2026-05-21T23:08:37.830961Z"
+++

## Spec

### Problem

`apm init` calls `update_claude_settings()` and `update_user_claude_settings()` (in `apm/src/cmd/init.rs`) to seed `.claude/settings.json` with the `APM_ALLOW_ENTRIES` allow-list that every worker needs. Two bugs prevent this from working reliably.

First, `update_claude_settings` passes `create_if_missing: false` to `update_settings_json` (line 283). If `.claude/` exists but `settings.json` does not, `update_settings_json` returns early at line 218 — the file is never created and the allow-list is never written.

Second, `update_settings_json` prompts the user `[y/N]` before writing (lines 232–244). In any non-TTY context — CI pipelines, headless worker spawning — `read_line` gets EOF and the function prints "Skipped." without writing anything. Workers spawned by `apm start --spawn` therefore hit the permission gate on every `apm spec`, `apm state`, and `apm show` call. Because `apm state <id> blocked` is also gated, the worker cannot even self-report the failure — a recursive trap.

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
| 2026-05-15T01:21Z | — | new | philippe|philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:08Z | groomed | in_design | philippepascal |