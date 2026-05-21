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

- [ ] `apm init --yes` on a repo with `.claude/` present but no `settings.json` creates `.claude/settings.json` containing all `APM_ALLOW_ENTRIES` under `permissions.allow` without prompting.
- [ ] `apm init --yes` on a repo with an existing `.claude/settings.json` that is missing some entries merges the missing entries in without duplicating entries that are already present.
- [ ] `apm init --yes` on a repo with no `.claude/` directory does not create `.claude/` or `settings.json`, and exits zero.
- [ ] `apm init` (no `--yes`) on a non-TTY stdin with `.claude/settings.json` absent still creates and seeds the file (non-interactive path does not prompt).
- [ ] `apm init --yes` updates `~/.claude/settings.json` with `APM_USER_ALLOW_ENTRIES` without prompting.
- [ ] `apm init --yes` prints `Updated .claude/settings.json` when the project file was created or modified, and prints `Updated ~/.claude/settings.json` when the user file was modified.
- [ ] `apm init --no-claude` still suppresses all settings.json writes even when `--yes` is passed.

### Out of scope

- Backfilling repos that ran a pre-fix `apm init` (migration command is a future concern).\n- Touching `settings.local.json` — that file is per-engineer and must not be written by `apm init`.\n- Creating the `.claude/` directory if it does not already exist.\n- Changing the interactive TTY flow — when stdin is a TTY and `--yes` is not passed, the existing [y/N] prompt is preserved.

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