+++
id = "4b663160"
title = "Implement apm epic new command"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "philippepascal"
branch = "ticket/4b663160-implement-apm-epic-new-command"
created_at = "2026-04-01T21:55:06.350633Z"
updated_at = "2026-04-02T00:43:25.666912Z"
+++

## Spec

### Problem

There is currently no way to create an epic. An epic is a git branch (`epic/<id>-<slug>`) — no separate file format needed. Without a command to create one, the entire epics workflow cannot be started.

The full design is in `docs/epics.md` (§ Commands — `apm epic new`). The command must:
1. Generate an 8-hex-char short ID
2. Slugify the title
3. Fetch `origin/main`, create `epic/<id>-<slug>` from its HEAD
4. Optionally commit an `EPIC.md` file (title as H1) to establish the branch as diverged from main
5. Push with `-u origin`
6. Print the branch name

The `apm epic` subcommand group does not yet exist and must be wired into the CLI.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
