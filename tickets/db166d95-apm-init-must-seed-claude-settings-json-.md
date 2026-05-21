+++
id = "db166d95"
title = "apm init must seed .claude/settings.json with worker-essential allow-list"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db166d95-apm-init-must-seed-claude-settings-json-"
created_at = "2026-05-15T01:21:17.353568Z"
updated_at = "2026-05-21T22:59:27.633946Z"
+++

## Spec

### Problem

`apm init` already has logic to seed `.claude/settings.json` — in the CLI layer at `apm/src/cmd/init.rs`:

- `update_claude_settings()` (line 273) targets the **project** `.claude/settings.json` with the `APM_ALLOW_ENTRIES` list (lines 138-167), which already includes every worker-essential command (`apm spec *`, `apm state *`, `apm show *`, `apm new *`, `apm set *`, etc.).
- `update_user_claude_settings()` (line 287) targets the **user** `~/.claude/settings.json` with `APM_USER_ALLOW_ENTRIES`.

But there are two reasons workers still crash with permission gates after a normal `apm init`:

1. **Project settings.json is only updated when it already exists.** `update_claude_settings` passes `create_if_missing: false` (line 283), so `update_settings_json` returns early at line 218 if the file is absent. A repo that already uses Claude Code (i.e. has a `.claude/` directory) but happens not to have `settings.json` yet never gets the project allow-list.

2. **Both helpers prompt interactively for [y/N] confirmation** (lines 232-244). In any non-TTY context the prompt is skipped via `read_line` returning empty/EOF, and the entries are not added. Also, a user typing anything other than `y` gets `Skipped.` and the file is left alone.

Reproduction: in this repo right now, `.claude/settings.json` does not exist, and `.claude/settings.local.json` carries only my conversation's session-specific entries (no `apm spec *` or `apm state *`). Workers spawned today on tickets `ba121f45` and `996fef40` both crashed because the apm-essential commands hit the permission gate, and the graceful-exit path (`apm state <id> blocked`) is also gated — a recursive trap.

Acceptance:
- `update_claude_settings` creates `.claude/settings.json` **only when the `.claude/` directory already exists**. Do not create the `.claude/` directory itself — that would silently opt non-Claude-Code projects into the Claude Code config tree.
- When `.claude/` is absent, skip the project-level settings update entirely (no warning needed; user-level fallback at `~/.claude/settings.json` still happens).
- When `.claude/` is present but `settings.json` is missing, create it with the full `APM_ALLOW_ENTRIES`.
- When `.claude/settings.json` already exists, merge in the missing entries (current behaviour).
- Add a `--yes`/non-interactive flag to `apm init` that adds the entries without prompting, so CI and `apm init` in headless environments do the right thing.
- After the change, a fresh `git init && mkdir .claude && apm init --yes` produces a `.claude/settings.json` with all `APM_ALLOW_ENTRIES` and the worker can spawn without permission crashes.
- A repo with no `.claude/` directory is unchanged by `apm init` (other than the existing CLAUDE.md / .apm/ writes).

Out of scope:
- Backfill of existing repos that ran a pre-fix `apm init`. A migration command can ship later if wanted.
- Modifying `settings.local.json` — that file is per-engineer and should never be touched by `apm init`.
- Creating the `.claude/` directory.

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
