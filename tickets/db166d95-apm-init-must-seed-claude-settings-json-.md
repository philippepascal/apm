+++
id = "db166d95"
title = "apm init must seed .claude/settings.json with worker-essential allow-list"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db166d95-apm-init-must-seed-claude-settings-json-"
created_at = "2026-05-15T01:21:17.353568Z"
updated_at = "2026-05-15T01:21:17.353568Z"
+++

## Spec

### Problem

`apm init` already writes `CLAUDE.md` to point at `agents.md` (`apm-core/src/init.rs::ensure_claude_md`), but it does **not** touch `.claude/settings.json`. Without that file, every `apm start --spawn` in a fresh repo hits Claude Code's permission gate — the worker tries to call `apm spec`, `apm state`, `apm show` etc., none of which are auto-allowed, and stalls. The 'graceful exit' path (`apm state <id> blocked`) hits the same gate, so the worker can't even self-escape and ends up marked `crashed` in `apm workers`.

Reproduction (just hit it twice in a row on tickets ba121f45 and 996fef40): worker log fills with `'This command requires approval'` / `'haven't granted it yet'`, then dies.

Acceptance:
- `apm init` writes a shared `.claude/settings.json` with the worker-essential commands allowed: `Bash(apm spec *)`, `Bash(apm state *)`, `Bash(apm show *)`, `Bash(apm new *)`, `Bash(apm set *)`, `Bash(apm next *)`, `Bash(apm list *)`, `Bash(apm sync *)`, `Bash(apm assign *)`, `Bash(apm version *)`, `Bash(apm validate *)`, plus `Write(<ticket-worktree>/**)` patterns the worker needs.
- Idempotent: if `.claude/settings.json` already exists, merge in the missing entries (do not overwrite user additions).
- The set of patterns lives next to `default_agents_md()` etc. so it's discoverable and editable.

Out of scope:
- `.claude/settings.local.json` is engineer-local; do not touch it.
- A migration command to backfill existing repos (separate ticket if wanted).

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
