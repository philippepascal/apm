+++
id = "12e947b1"
title = "apm init Claude settings allow-list missing commands"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12e947b1-apm-init-claude-settings-allow-list-miss"
created_at = "2026-04-24T06:28:47.480554Z"
updated_at = "2026-04-24T07:19:57.933406Z"
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

- Changes to the apm init command behaviour or flags\n- Adding non-apm entries to the allow-lists (git commands are out of scope)\n- Dynamic generation of allow-list entries from the binary's clap command registry\n- Adding apm init itself to either allow-list (it is a destructive setup command that should remain user-prompted)\n- Auditing or modifying allow-lists in any project that has already run apm init (existing settings.json files are not updated retroactively)

### Approach

Single file change: `apm/src/cmd/init.rs`.

**In `APM_ALLOW_ENTRIES` (around line 121–136):**

1. Remove the line `"Bash(apm take*)",` — this subcommand does not exist; `start` is already present.
2. Append the following entries (order within the list does not matter):
   ```
   "Bash(apm help*)",
   "Bash(apm review*)",
   "Bash(apm close*)",
   "Bash(apm assign*)",
   "Bash(apm validate*)",
   "Bash(apm work*)",
   "Bash(apm move*)",
   "Bash(apm archive*)",
   "Bash(apm clean*)",
   "Bash(apm workers*)",
   "Bash(apm epic*)",
   "Bash(apm register*)",
   "Bash(apm sessions*)",
   "Bash(apm revoke*)",
   "Bash(apm version*)",
   ```

**In `APM_USER_ALLOW_ENTRIES` (around line 140–156):**

1. Remove the line `"Bash(apm take*)",` — same reason as above.
2. Add `"Bash(apm spec*)",` — currently missing from user-level list only.
3. Append the same 15 new entries listed above.

**Glob pattern convention:** Use `"Bash(apm <sub>*)"` (no space before `*`) for commands that take no args or where glob-matching the name alone is sufficient (e.g. `apm version*`). Use `"Bash(apm <sub> *)"` (space before `*`) for commands that require an argument (e.g. `apm review *`). Follow the existing pattern in the file: commands that need args already use `"Bash(apm set *)"` style. For the new entries, commands like `review`, `close`, `assign`, `work`, `move`, `archive`, `clean`, `epic`, `register`, `revoke`, `sessions` take arguments — use a trailing space before `*`. Commands like `help`, `validate`, `workers`, `version` may be invoked bare — use no space. Match the style of neighbouring existing entries.

**No other files change.** The constants are `&[&str]` slices used directly by the init logic that writes settings.json; adding entries here is sufficient.

After editing, run the existing init integration tests (if any) to confirm the new entries appear in test output. No new tests are strictly required for this change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:16Z | groomed | in_design | philippepascal |
| 2026-04-24T07:19Z | in_design | specd | claude-0424-0716-4bc8 |
