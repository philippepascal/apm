+++
id = "a76f3a3d"
title = "apm init: seed code-editing perms in .claude/settings.json"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a76f3a3d-apm-init-seed-code-editing-perms-in-clau"
created_at = "2026-05-17T19:52:57.296867Z"
updated_at = "2026-05-21T22:59:31.301179Z"
depends_on = ["db166d95"]
+++

## Spec

### Problem

`apm init` already has an `APM_ALLOW_ENTRIES` list (`apm/src/cmd/init.rs:138-167`) covering every worker-essential `apm` subcommand. Sibling ticket `db166d95` covers the gap where the file is never created when `.claude/` exists but `settings.json` doesn't.

This ticket covers the **content** of that list. Today it only carries `Bash(apm …)` patterns. That unblocks `apm spec`, `apm state` etc., but a real worker doing implementation work needs more: the `Edit` tool to modify code files, `Bash(cargo *)` to run tests, `Bash(git -C *)` for git ops in its worktree, and a handful of read helpers (`grep`, `rg`, `find`, `cat`, etc.).

Reproduction (today, 2026-05-15): ticket `996fef40` is a 5-line surgical fix to `apm/src/cmd/work.rs`. Worker spawned successfully (`Bash(apm …)` was allowed), wrote a correct spec, then transitioned to `in_progress` and immediately stalled because the `Edit` tool was not allow-listed. The graceful-exit path worked — the worker wrote an Open question and transitioned to `blocked` — but the bare-minimum allow-list traps every implementation ticket at the same step.

The set we just added manually to this repo's `.claude/settings.json` (alongside the existing apm allow-list) is a reasonable starting point:

- `Edit`, `Write` — code editing
- `Bash(cargo *)` — Rust toolchain (project-specific; see decision below)
- `Bash(git -C *)` — git operations from worktree
- `Bash(python3 *)` — scripting (sometimes used by workers as an Edit alternative)
- `Bash(ls *)`, `Bash(rg *)`, `Bash(grep *)`, `Bash(find *)`, `Bash(cat *)`, `Bash(head *)`, `Bash(tail *)`, `Bash(wc *)`, `Bash(sort *)`, `Bash(uniq *)`, `Bash(diff *)`, `Bash(which *)` — read helpers
- `Bash(sed *)`, `Bash(awk *)` — text manipulation
- `Bash(mv *)`, `Bash(cp *)`, `Bash(rm /tmp/*)`, `Bash(mkdir -p /tmp/*)` — file ops scoped to safe areas
- `Bash(echo *)`, `Bash(test *)`, `Bash(true)`, `Bash(false)` — shell building blocks

Design decision the implementer must make: should `apm init` ship a **language-agnostic** baseline (Edit + read helpers + git) and leave language-specific entries like `Bash(cargo *)`, `Bash(npm *)`, `Bash(pytest *)` to the supervisor, OR should it detect the project type (presence of Cargo.toml, package.json, etc.) and append the right toolchain bash entries? Option A is simpler; option B is more helpful. Either is defensible.

Acceptance:
- `APM_ALLOW_ENTRIES` (or a follow-up constant `APM_CODE_EDIT_ENTRIES` merged into the seed) covers the language-agnostic baseline at minimum: `Edit`, `Write`, `Bash(git -C *)`, and the read helpers listed above.
- Decision recorded in Approach about whether language toolchains (`cargo`, `npm`, `pytest`, etc.) are seeded automatically based on detected project files, or left to the supervisor to add by hand.
- After this ticket + `db166d95`, running `apm init` in a fresh repo with `.claude/` present produces a `settings.json` that lets a worker complete a typical implementation ticket without permission stalls.

Depends on `db166d95` so the writer and reviewer don't have to think about the create-vs-update logic at the same time.

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
