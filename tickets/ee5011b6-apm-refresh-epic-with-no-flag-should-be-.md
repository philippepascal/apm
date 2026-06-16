+++
id = "ee5011b6"
title = "apm refresh-epic with no flag should be interactive and propose several merge action"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ee5011b6-apm-refresh-epic-with-no-flag-should-be-"
created_at = "2026-06-16T18:06:05.166190Z"
updated_at = "2026-06-16T20:35:09.922445Z"
+++

## Spec

### Problem

When `apm refresh-epic <id>` is run with no action flag (`--merge`, `--pr`, or `--auto`), the command prints a one-line status message and exits without doing anything. On an interactive terminal this is unhelpful: the user already knows there are commits to pull in, and now must re-type the command with the right flag to act on that information.

The fix is to turn the no-flag path into an interactive prompt when stdout is a terminal. The command should show the same status it already computes, then offer a numbered menu of the same actions the flags expose, read the user's choice, and execute it. Non-interactive callers (pipes, headless agents) keep the current print-and-exit behaviour so no scripted usage breaks.

### Acceptance criteria

- [x] `apm refresh-epic <id>` with no flag, on a terminal, when the epic branch is ahead of the default branch, prints the status line and then displays a numbered menu of actions.
- [x] The menu offers at least three actions: merge locally, open / update PR, auto (merge if clean, fall back to PR).
- [x] The menu includes a "skip" option that exits without performing any action.
- [x] Choosing "merge locally" has the same outcome as passing `--merge`.
- [x] Choosing "open / update PR" has the same outcome as passing `--pr`.
- [x] Choosing "auto" has the same outcome as passing `--auto`.
- [x] When the epic branch is already up to date, `apm refresh-epic <id>` with no flag prints "epic branch is up to date" and exits — no menu is shown.
- [x] When stdout is not a terminal (pipe or headless), `apm refresh-epic <id>` with no flag retains the current behaviour: print status and exit without prompting.

### Out of scope

- The push-after-merge prompt (`Push refreshed epic to origin? [Y/n]`) is unchanged by this ticket.
- No new flags are added; the existing `--merge`, `--pr`, `--auto`, `--push`, `--no-push` flags are unchanged.
- No changes to non-interactive (headless / piped) behaviour.
- No changes to `apm epic submit` or any other subcommand.

### Approach

All changes are in `apm/src/cmd/epic.rs`, `run_refresh_epic`. No new files; no changes to `apm-core`.

#### Make action flags mutable

Shadow the incoming `merge`, `pr`, and `auto_mode` parameters as mutable locals at the top of the function:

```rust
let mut merge = merge;
let mut pr = pr;
let mut auto_mode = auto_mode;
```

#### Replace the early-return block with conditional interactive prompt

The current block:

```rust
let acting = merge || pr || auto_mode;
if !acting {
    // print status
    return Ok(());
}
```

Replace with logic that, when `!acting && std::io::stdout().is_terminal()` and `status.ahead > 0`, shows a menu and reads a choice instead of returning. The revised flow:

1. If `status.ahead == 0`: print "up to date" and `return Ok(())` (same as now, regardless of terminal).
2. Otherwise print the status line (same text as now).
3. If NOT terminal: `return Ok(())` (non-interactive exit, same as now).
4. If terminal: display the menu, read a single line from stdin, and map the choice:
   - `1` → `merge = true`
   - `2` → `pr = true`
   - `3` → `auto_mode = true`
   - `4` or anything else / empty → `return Ok(())` (skip)

No helper function is needed; the prompt is a single `print!` + `flush` + `read_line` in-lined directly, matching the pattern already used in `run_refresh_epic` for the push prompt.

#### Menu text

```
What would you like to do?
  [1] Merge locally
  [2] Open / update PR
  [3] Auto (merge if clean, fall back to PR)
  [4] Skip
Choice [1-4]: 
```

After this block, fall through into the existing quiescence check and merge/PR logic unchanged — `merge`, `pr`, and `auto_mode` are now set correctly.

#### Tests

Add one integration test in `apm/tests/integration.rs` (or an existing `refresh_epic` test module if present) covering the non-terminal path: confirm that with no flags and no terminal, the command prints the status line and exits 0 without attempting a merge.

Interactive-terminal coverage is manual; the non-terminal path is the automatable regression guard.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:06Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:09Z | groomed | in_design | philippepascal |
| 2026-06-16T18:11Z | in_design | specd | claude |
| 2026-06-16T20:24Z | specd | ready | philippepascal |
| 2026-06-16T20:35Z | ready | in_progress | philippepascal |