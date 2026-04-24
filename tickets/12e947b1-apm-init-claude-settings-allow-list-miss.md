+++
id = "12e947b1"
title = "apm init Claude settings allow-list missing commands"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12e947b1-apm-init-claude-settings-allow-list-miss"
created_at = "2026-04-24T06:28:47.480554Z"
updated_at = "2026-04-24T07:16:49.824155Z"
+++

## Spec

### Problem

`APM_ALLOW_ENTRIES` and `APM_USER_ALLOW_ENTRIES` in `apm/src/cmd/init.rs` (lines 121–156) define the commands written into `.claude/settings.json` and `~/.claude/settings.json` respectively when `apm init` is run. These allow-lists exist so Claude Code does not open a permission prompt for routine apm invocations during a ticket session.

The allow-lists are significantly out of date relative to the command set the binary now exposes. An audit against all `apm/src/cmd/*.rs` files and the top-level CLI enum in `main.rs` reveals 15 subcommands that trigger permission prompts but are not whitelisted: `help`, `review`, `close`, `assign`, `validate`, `work`, `move`, `archive`, `clean`, `workers`, `epic`, `register`, `sessions`, `revoke`, and `version`. The user-level list has an additional gap: `spec` is whitelisted in the project list but absent from the user list.

Both lists also contain the ghost entry `"Bash(apm take*)"` — `take` was renamed to `start`, which is already whitelisted; the dead entry has no effect but indicates the lists have drifted.

Every missing command has been observed to trigger a mid-session prompt during normal ticker workflow use, interrupting automated agent runs.

### Acceptance criteria

- [ ] `APM_ALLOW_ENTRIES` contains a glob entry for every subcommand the binary exposes: `apm help*`, `apm review*`, `apm close*`, `apm assign*`, `apm validate*`, `apm work*`, `apm move*`, `apm archive*`, `apm clean*`, `apm workers*`, `apm epic*`, `apm register*`, `apm sessions*`, `apm revoke*`, `apm version*`
- [ ] `APM_USER_ALLOW_ENTRIES` contains `"Bash(apm spec*)"` (currently absent from the user-level list)
- [ ] `APM_USER_ALLOW_ENTRIES` contains the same 15 new subcommand globs listed above
- [ ] The ghost entry `"Bash(apm take*)"` is removed from `APM_ALLOW_ENTRIES`
- [ ] The ghost entry `"Bash(apm take*)"` is removed from `APM_USER_ALLOW_ENTRIES`
- [ ] `apm init` is not added to either allow-list
- [ ] Running `apm init` in a fresh directory and inspecting the generated `.claude/settings.json` shows all newly added entries present under `permissions.allow`
- [ ] Running `apm init --user` and inspecting `~/.claude/settings.json` shows all newly added entries present under `permissions.allow`

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
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:16Z | groomed | in_design | philippepascal |