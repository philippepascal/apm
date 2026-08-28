+++
id = "0c065989"
title = "apm refresh-epic needs to be moved under apm epic refresh for consistency"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0c065989-apm-refresh-epic-needs-to-be-moved-under"
created_at = "2026-08-28T00:50:30.208076Z"
updated_at = "2026-08-28T07:24:17.681175Z"
+++

## Spec

### Problem

`apm refresh-epic <id>` is a top-level command, even though every other
epic-scoped operation (`new`, `submit`, `close`, `list`, `show`, `set`) lives
under `apm epic <subcommand>`. This is the only epic operation that breaks the
`apm epic ...` convention, which makes the CLI surface harder to discover
(`apm epic --help` doesn't mention it) and inconsistent with the rest of the
command tree documented in `apm main.rs`'s help template and `apm help
commands`.

The command should be relocated to `apm epic refresh <id>` so all epic
operations are grouped under one subcommand namespace, with identical
behaviour and flags. No backward-compatible alias is kept for the old
top-level name, per the project's no-shim convention.

### Acceptance criteria

- [ ] `apm epic refresh <id>` (no flags) prints the ahead-count / clean-vs-conflicted status and, when stdout is a terminal, prompts for merge/PR/auto/skip — identical to the current `apm refresh-epic <id>` no-flag behaviour
- [ ] `apm epic refresh <id> --merge` performs a local merge of the default branch into the epic branch, identical to current `apm refresh-epic --merge`
- [ ] `apm epic refresh <id> --pr` opens or updates a PR from the default branch into the epic branch, identical to current `apm refresh-epic --pr`
- [ ] `apm epic refresh <id> --auto` merges locally when clean and falls back to a PR on conflict, identical to current `apm refresh-epic --auto`
- [ ] `apm epic refresh <id> --merge --push` and `--merge --no-push` control the post-merge push exactly as `apm refresh-epic` did
- [ ] `apm refresh-epic <id>` no longer exists — running it fails with clap's unknown-subcommand error
- [ ] `apm epic --help` lists `refresh` alongside `new`, `submit`, `close`, `list`, `show`, `set`
- [ ] `apm help commands` no longer lists `refresh-epic` as a top-level command, and lists `refresh` nested under `epic`
- [ ] `cargo test --workspace` passes, including integration tests exercising the relocated command under `apm epic refresh`

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
| 2026-08-28T00:50Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:24Z | groomed | in_design | philippepascal |