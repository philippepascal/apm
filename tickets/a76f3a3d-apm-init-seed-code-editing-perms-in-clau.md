+++
id = "a76f3a3d"
title = "apm init: seed code-editing perms in .claude/settings.json"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a76f3a3d-apm-init-seed-code-editing-perms-in-clau"
created_at = "2026-05-17T19:52:57.296867Z"
updated_at = "2026-05-21T23:10:43.621907Z"
depends_on = ["db166d95"]
+++

## Spec

### Problem

`apm init` seeds `APM_ALLOW_ENTRIES` into `.claude/settings.json` so workers can invoke `apm` subcommands without permission prompts. That list (`apm/src/cmd/init.rs:138-167`) covers only `Bash(apm …)` patterns. Sibling ticket `db166d95` fixes the create-vs-update gap so the file is written when `.claude/` is present but `settings.json` is not; this ticket covers the *content* of that file.

The gap is concrete: ticket `996fef40` triggered a worker that successfully called `apm spec` and `apm state` (both allowed), then stalled the moment it tried to open a file with the `Edit` tool — not on the list. The worker's graceful-exit path worked, but every implementation ticket hits the same wall. Workers also need read helpers (`grep`, `find`, `cat`, etc.), text manipulation (`sed`, `awk`), safe file ops (`mv`, `cp`, `/tmp` writes), and git ops inside their worktree (`git -C …`). Language-specific toolchain commands (`cargo`, `npm`, `python3`) complete the picture for build and test steps.

The desired end state: after `db166d95` + this ticket, `apm init` in a project with `.claude/` present writes a `settings.json` that lets a worker edit source files, run the project's test suite, and complete a typical implementation ticket without hitting a permission prompt.

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
| 2026-05-17T19:52Z | — | new | philippe|philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:10Z | groomed | in_design | philippepascal |